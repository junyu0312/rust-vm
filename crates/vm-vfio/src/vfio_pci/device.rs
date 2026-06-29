use std::iter;
use std::sync::Arc;

use vfio_bindings::bindings::vfio::VFIO_IRQ_INFO_EVENTFD;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR5_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_CONFIG_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_READ;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_WRITE;
use vm_core::device::Device;
use vm_core::virtualization::irq_allocator::IrqAllocator;
use vm_core::virtualization::vm::HypervisorVm;
use vm_pci::device::capability::PciCapId;
use vm_pci::device::capability::msi::PCI_MSI_FLAGS_64BIT;
use vm_pci::device::capability::msi::PCI_MSI_FLAGS_MASKBIT;
use vm_pci::device::capability::msi::PCI_MSI_FLAGS_QMASK;
use vm_pci::device::capability::msi::PciMsiCap;
use vm_pci::device::capability::msi::PciMsiCap64;
use vm_pci::device::capability::msi::PciMsiCap64Mask;
use vm_pci::device::capability::msi::PciMsiCapMask;
use vm_pci::device::capability::msi::PciMsiMmc;
use vm_pci::device::capability::msix::MsixEntry;
use vm_pci::device::capability::msix::PCI_MSIX_ENTRY_CTRL_MASKBIT;
use vm_pci::device::capability::msix::PCI_MSIX_FLAGS_QSIZE;
use vm_pci::device::capability::msix::PCI_MSIX_PBA_OFFSET;
use vm_pci::device::capability::msix::PCI_MSIX_TABLE_BIR;
use vm_pci::device::capability::msix::PCI_MSIX_TABLE_OFFSET;
use vm_pci::device::capability::msix::PciMsixCap;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_32;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_64;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_MASK;
use vm_pci::types::bar::PCI_BASE_ADDRESS_SPACE;
use vm_pci::types::bar::PciBarInfo;
#[cfg(target_arch = "x86_64")]
use vm_pci::types::bar::pci_io_bar;
use vm_pci::types::bar::pci_mmio_32_bar;
use vm_pci::types::bar::pci_mmio_64_bar;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::PciConfigurationSpace;
use vm_pci::types::configuration_space::capability::StandardCapability;
use vm_pci::types::configuration_space::command::PciCommand;
use vm_pci::types::configuration_space::header::CommonHeaderOffset;
use vm_pci::types::configuration_space::header::PciHeaderType;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::configuration_space::status::PciStatus;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;
use vm_utils::range_allocator::RangeAllocator;
use vmm_sys_util::eventfd::EventFd;
use zerocopy::FromBytes;
use zerocopy::IntoBytes;

use crate::error::Error;
use crate::error::Result;
use crate::vfio::device::VfioDevice;
use crate::vfio_pci::function::VfioPciFunction;
use crate::vfio_pci::interrupt::VfioInterruptInfo;
use crate::vfio_pci::interrupt::VfioInterruptManager;
use crate::vfio_pci::interrupt::intx::VfioIntx;
use crate::vfio_pci::interrupt::intx::VfioIntxInfo;
use crate::vfio_pci::interrupt::msi::VfioMsi;
use crate::vfio_pci::interrupt::msi::VfioMsiInfo;
use crate::vfio_pci::interrupt::msix::VfioMsix;
use crate::vfio_pci::interrupt::msix::VfioMsixInfo;

const DEBUG_ENABLE_MSIX: bool = false;
const DEBUG_ENABLE_MSI: bool = true;
const DEBUG_ENABLE_INTX: bool = true;

fn setup_interrupt_capability(
    vm: &dyn HypervisorVm,
    vfio_device: Arc<VfioDevice>,
    irq_allocator: &mut IrqAllocator,
    raw: &PciConfigurationSpace,
    cfg: &mut ConfigurationSpace,
) -> Result<(VfioInterruptInfo, VfioInterruptManager)> {
    let mut msix = None;
    let mut msix_info = None;
    if DEBUG_ENABLE_MSIX && let Some(offset) = raw.find_cap(PciCapId::MsiX as u8) {
        let cap = PciMsixCap::ref_from_bytes(
            &raw.as_bytes()[offset as usize..offset as usize + size_of::<PciMsixCap>()],
        )
        .map_err(|_| Error::ParseMsiX)?;

        let vectors = (cap.ctrl & PCI_MSIX_FLAGS_QSIZE) + 1;

        let irq_info = vfio_device
            .get_msix_irq_info()
            .ok_or(Error::PrepareIrq("Failed to get msi-x info".into()))?;

        if irq_info.count == 0 {
            return Err(Error::PrepareIrq("msi-x count is zero".into()));
        }

        if irq_info.count != vectors as u32 {
            return Err(Error::PrepareIrq("msi-x count inconsistent".into()));
        }

        if irq_info.flags & VFIO_IRQ_INFO_EVENTFD == 0 {
            return Err(Error::PrepareIrq("msi-x does not support eventfd".into()));
        }

        let mut table = (0..vectors)
            .map(|_| MsixEntry::default())
            .collect::<Vec<_>>();
        for entry in table.iter_mut() {
            entry.control = PCI_MSIX_ENTRY_CTRL_MASKBIT;
        }
        let table_bar = (cap.table_offset & PCI_MSIX_TABLE_BIR) as u8;
        let table_offset = cap.table_offset & PCI_MSIX_TABLE_OFFSET;
        let table_len = table.as_bytes().len();

        let pba = vec![0; vectors.div_ceil(8) as usize];
        let pba_bar = (cap.pba_offset & PCI_MSIX_TABLE_BIR) as u8;
        let pba_offset = cap.pba_offset & PCI_MSIX_PBA_OFFSET;
        let pba_len = pba.as_bytes().len();

        let event_fds = (0..vectors)
            .map(|_| EventFd::new(0))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let cap = StandardCapability::from(PciMsixCap::new(
            vectors,
            table_bar,
            table_offset,
            pba_bar,
            pba_offset,
        ));
        let cap_len = cap.cap_len();
        let cap_offset = cfg.alloc_capability(cap)?;

        msix_info = Some(VfioMsixInfo {
            event_fds,
            table_bar,
            table_offset,
            table_len: table_len.try_into().unwrap(),
            pba_bar,
            pba_offset,
            pba_len: pba_len.try_into().unwrap(),
            cap_offset_range: cap_offset as u16..cap_offset as u16 + cap_len as u16,
        });
        msix = Some(VfioMsix {
            table,
            pba,
            enabled: false,
        });
    }

    let mut msi_info = None;
    let mut msi = None;
    if DEBUG_ENABLE_MSI && let Some(offset) = raw.find_cap(PciCapId::Msi as u8) {
        let ctrl = u16::from_le_bytes(
            raw.as_bytes()[offset as usize + 2..offset as usize + 4]
                .try_into()
                .unwrap(),
        );
        let mmc = PciMsiMmc::from_repr(((ctrl & PCI_MSI_FLAGS_QMASK) >> 1) as u8)
            .ok_or(Error::ParseMsi)?;
        let vectors = mmc.vectors();

        let irq_info = vfio_device
            .get_msi_irq_info()
            .ok_or(Error::PrepareIrq("Failed to get msi info".into()))?;

        if irq_info.count == 0 {
            return Err(Error::PrepareIrq("msi count is zero".into()));
        }

        if irq_info.count != vectors as u32 {
            return Err(Error::PrepareIrq("msi count inconsistent".into()));
        }

        if irq_info.flags & VFIO_IRQ_INFO_EVENTFD == 0 {
            return Err(Error::PrepareIrq("msi does not support eventfd".into()));
        }

        let bit64 = ctrl & PCI_MSI_FLAGS_64BIT != 0;
        let mask = ctrl & PCI_MSI_FLAGS_MASKBIT != 0;
        let cap_offset;
        let cap_len;

        match (bit64, mask) {
            (false, false) => {
                let cap: StandardCapability = PciMsiCap::new(mmc).into();
                cap_len = cap.cap_len();
                cap_offset = cfg.alloc_capability(cap)?;
            }
            (true, false) => {
                let cap: StandardCapability = PciMsiCap64::new(mmc).into();
                cap_len = cap.cap_len();
                cap_offset = cfg.alloc_capability(cap)?;
            }
            (false, true) => {
                let cap: StandardCapability = PciMsiCapMask::new(mmc).into();
                cap_len = cap.cap_len();
                cap_offset = cfg.alloc_capability(cap)?;
            }
            (true, true) => {
                let cap: StandardCapability = PciMsiCap64Mask::new(mmc).into();
                cap_len = cap.cap_len();
                cap_offset = cfg.alloc_capability(cap)?;
            }
        }

        let irqrd = vec![false; mmc.vectors() as usize];
        let event_fds = (0..mmc.vectors())
            .map(|_| EventFd::new(0))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        msi_info = Some(VfioMsiInfo {
            event_fds,
            bit64,
            mask,
            cap_offset_range: cap_offset as u16..cap_offset as u16 + cap_len as u16,
        });
        msi = Some(VfioMsi {
            irqrd,
            enabled: false,
        });
    }

    let mut intx = None;
    let mut intx_info = None;
    if DEBUG_ENABLE_INTX
        && let Some(irq_info) = vfio_device.get_intx_irq_info()
        && irq_info.count != 0
    {
        assert_eq!(irq_info.count, 1);

        let raw_header = raw.as_header::<Type0Header>();
        if raw_header.interrupt_pin != InterruptPin::Empty as u8 {
            let header = cfg.as_header_mut::<Type0Header>();

            let interrupt_pin =
                InterruptPin::from_repr(raw_header.interrupt_pin).ok_or(Error::ParseIntx)?;
            header.interrupt_pin = interrupt_pin as u8;

            let gsi = irq_allocator
                .alloc()
                .map_err(|_| Error::AllocIrq)?
                .try_into()
                .unwrap();
            header.interrupt_line = gsi;

            if irq_info.flags & VFIO_IRQ_INFO_EVENTFD == 0 {
                return Err(Error::PrepareIrq("intx does not support eventfd".into()));
            }

            let active_fd = EventFd::new(0).map_err(|err| Error::PrepareIrq(err.into()))?;
            let deactive_fd = EventFd::new(0).map_err(|err| Error::PrepareIrq(err.into()))?;

            vm.set_irqfd_with_resample(&active_fd, &deactive_fd, gsi as u32)
                .map_err(|err| Error::PrepareIrq(err.into()))?;

            vfio_device.enable_intx(&active_fd)?;
            vfio_device.set_intx_resample_fd(&deactive_fd)?;

            intx_info = Some(VfioIntxInfo {
                gsi: gsi as u32,
                trigger_fd: active_fd,
                resample_fd: deactive_fd,
                pin: interrupt_pin,
                line: header.interrupt_line,
            });
            intx = Some(VfioIntx { enabled: true });
        }
    }

    let interrupt_info = VfioInterruptInfo {
        intx: intx_info,
        msi: msi_info,
        msix: msix_info,
    };
    let interrupt_manager = VfioInterruptManager { intx, msi, msix };

    Ok((interrupt_info, interrupt_manager))
}

pub struct VfioPciDevice {
    name: String,
    function: VfioPciFunction,
}

impl VfioPciDevice {
    pub fn new(
        name: String,
        vm: Arc<dyn HypervisorVm>,
        #[cfg(target_arch = "x86_64")] pci_io_window_allocator: &mut RangeAllocator<u16>,
        pci_mmio_window_allocator: &mut RangeAllocator<u64>,
        irq_allocator: &mut IrqAllocator,
        vfio_device: VfioDevice,
    ) -> Result<Self> {
        let vfio_device = Arc::new(vfio_device);
        vfio_device.reset()?;

        // Get raw header from device
        let raw_configuration_space = {
            let mut configuration_space = [0u8; 4096];

            let pci_config_region = vfio_device.get_region_info(VFIO_PCI_CONFIG_REGION_INDEX)?;
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_READ != 0);
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_WRITE != 0);

            let mut buf = vec![0; pci_config_region.size as usize];
            vfio_device.region_read(VFIO_PCI_CONFIG_REGION_INDEX, &mut buf, 0)?;

            configuration_space[0..buf.len()].copy_from_slice(&buf);

            PciConfigurationSpace::from_buf(configuration_space)
        };

        if PciHeaderType::from_repr(raw_configuration_space.as_common_header().header_type)
            .ok_or(Error::UnknownPciHeaderType)?
            != PciHeaderType::Device
        {
            return Err(Error::VfioPciDeviceIsNotEndpoint);
        }

        let raw_header = raw_configuration_space.as_header::<Type0Header>();

        // Prepare virtual common header for vfio device
        let mut configuration_space = ConfigurationSpace::default();
        {
            configuration_space.write(0, raw_configuration_space.as_bytes());
            let header = configuration_space.as_header_mut::<Type0Header>();

            // Clear fields
            header.common.command &= !PciCommand::IO.bits();
            header.common.command &= !PciCommand::MEMORY.bits();
            for index in VFIO_PCI_BAR0_REGION_INDEX..=VFIO_PCI_BAR5_REGION_INDEX {
                let index = index as usize;
                if raw_header.bar[index] & PCI_BASE_ADDRESS_SPACE == 0 {
                    header.bar[index] = raw_header.bar[index] & 0xf;
                } else {
                    header.bar[index] = raw_header.bar[index] & 0x3;
                }
            }
            header.expansion_rom_base_address = 0;
            header.interrupt_line = 0xff;
            header.interrupt_pin = InterruptPin::Empty as u8;
            header.common.status &= !(PciStatus::CapList as u16);
            header.cap_pointer = 0;
            configuration_space.as_bytes_mut()[CommonHeaderOffset::CapabilityStart as usize..]
                .fill(0);
        }

        let (interrupt_info, interrupt_manager) = setup_interrupt_capability(
            vm.as_ref(),
            vfio_device.clone(),
            irq_allocator,
            &raw_configuration_space,
            &mut configuration_space,
        )?;

        let mut bar_info = [const { None }; 6];
        {
            let raw_header = raw_configuration_space.as_header::<Type0Header>();
            let header = configuration_space.as_header_mut::<Type0Header>();

            for index in VFIO_PCI_BAR0_REGION_INDEX..=VFIO_PCI_BAR5_REGION_INDEX {
                let region = vfio_device.get_region_info(index)?;

                if region.size == 0 {
                    continue;
                }

                let len: usize = region.size.try_into().unwrap();
                let index = index as usize;
                let bar = raw_header.bar[index];

                let is_mmio = bar & PCI_BASE_ADDRESS_SPACE == 0;

                let resource = if is_mmio {
                    let bar_mem_type = bar & PCI_BASE_ADDRESS_MEM_TYPE_MASK;
                    let is_64bit = if bar_mem_type == PCI_BASE_ADDRESS_MEM_TYPE_32 {
                        false
                    } else if bar_mem_type == PCI_BASE_ADDRESS_MEM_TYPE_64 {
                        true
                    } else {
                        return Err(Error::InvalidMmioBarType(index, bar_mem_type));
                    };

                    let range = pci_mmio_window_allocator.alloc(len).unwrap();
                    if is_64bit {
                        let addr = range.start;
                        let low = (addr & 0xFFFF_FFFF) as u32;
                        let high = (addr >> 32) as u32;
                        header.bar[index] = pci_mmio_64_bar(low);
                        header.bar[index + 1] = high;
                    } else {
                        header.bar[index] = pci_mmio_32_bar(range.start.try_into().unwrap())
                    };

                    PciBarInfo::Mmio { is_64bit, len }
                } else {
                    #[cfg(target_arch = "x86_64")]
                    {
                        let range = pci_io_window_allocator.alloc(len).unwrap();
                        header.bar[index] = pci_io_bar(range.start);

                        PciBarInfo::Pio { len }
                    }

                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        unreachable!()
                    }
                };

                bar_info[index] = Some(resource);
            }
        }

        let function = VfioPciFunction::new(
            vm,
            raw_configuration_space,
            configuration_space,
            bar_info,
            vfio_device,
            interrupt_info,
            interrupt_manager,
        );

        Ok(VfioPciDevice { name, function })
    }
}

impl Device for VfioPciDevice {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl PciDevice for VfioPciDevice {
    fn get_function(&self, function: u8) -> Option<&dyn PciFunction> {
        if function == 0 {
            return Some(&self.function);
        }

        None
    }

    fn get_function_mut(&mut self, function: u8) -> Option<&mut dyn PciFunction> {
        if function == 0 {
            return Some(&mut self.function);
        }

        None
    }

    fn functions(&self) -> Box<dyn Iterator<Item = &(dyn PciFunction + '_)> + '_> {
        Box::new(iter::once(&self.function as &dyn PciFunction))
    }
}
