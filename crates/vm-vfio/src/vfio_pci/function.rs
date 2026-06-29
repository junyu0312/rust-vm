use std::sync::Arc;
use std::sync::Mutex;

use tracing::warn;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_CONFIG_REGION_INDEX;
use vm_core::virtualization::kvm::gsi_routing::get_kvm_sgi_routing_instance;
use vm_core::virtualization::vm::HypervisorVm;
use vm_pci::device::capability::msix::MsixEntry;
use vm_pci::device::capability::msix::PCI_MSIX_FLAGS_ENABLE;
use vm_pci::device::capability::msix::PciMsixCap;
use vm_pci::device::capability::msix::PciMsixCapOffset;
use vm_pci::types::bar::PciBarInfo;
use vm_pci::types::bar::address_of_bar;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::PciConfigurationSpace;
use vm_pci::types::configuration_space::command::PciCommand;
use vm_pci::types::configuration_space::header::CommonHeaderOffset;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::function::EcamUpdateCallback;
use vm_pci::types::function::EcamUpdateCallbackOps;
use vm_pci::types::function::PciFunction;
use vm_pci::types::function::PciFunctionArch;
use vm_pci::types::function::type0::Type0HeaderOffset;
use vm_pci::types::interrupt::InterruptMapEntry;
use zerocopy::FromBytes;
use zerocopy::IntoBytes;

use crate::vfio::device::VfioDevice;
use crate::vfio_pci::interrupt::VfioInterruptInfo;
use crate::vfio_pci::interrupt::VfioInterruptManager;
use crate::vfio_pci::interrupt::msix::VfioMsixInfo;

pub struct VfioPciFunction {
    vm: Arc<dyn HypervisorVm>,
    raw_configuration_space: PciConfigurationSpace,
    configuration_space: Mutex<ConfigurationSpace>,
    bars: [Option<PciBarInfo>; 6],
    device: Arc<VfioDevice>,
    interrupt_manager: Arc<Mutex<VfioInterruptManager>>,
    interrupt_info: VfioInterruptInfo,
}

impl VfioPciFunction {
    pub(crate) fn new(
        vm: Arc<dyn HypervisorVm>,
        raw_configuration_space: PciConfigurationSpace,
        configuration_space: ConfigurationSpace,
        bars: [Option<PciBarInfo>; 6],
        device: Arc<VfioDevice>,
        interrupt_info: VfioInterruptInfo,
        interrupt_manager: VfioInterruptManager,
    ) -> Self {
        VfioPciFunction {
            vm,
            raw_configuration_space,
            configuration_space: configuration_space.into(),
            bars,
            device,
            interrupt_info,
            interrupt_manager: Arc::new(Mutex::new(interrupt_manager)),
        }
    }

    fn read_msix_table(&self, offset: usize, buf: &mut [u8]) {
        let interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_ref().unwrap();

        buf.copy_from_slice(&msix.table.as_bytes()[offset..offset + buf.len()]);
    }

    fn write_msix_table(&self, msix_info: &VfioMsixInfo, offset: usize, buf: &[u8]) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_mut().unwrap();

        let vector = offset / size_of::<MsixEntry>();
        let vector = usize::try_from(vector).unwrap();

        let offset_within_entry = offset % size_of::<MsixEntry>();
        let offset_within_entry = usize::try_from(offset_within_entry).unwrap();

        let entry = &mut msix.table[vector];
        let ctrl_old = entry.control;

        entry.as_mut_bytes()[offset_within_entry..offset_within_entry + buf.len()]
            .copy_from_slice(buf);
        let ctrl_new = entry.control;

        if ctrl_old == ctrl_new {
            return;
        }

        // TODO: introduce sgi allocator
        let sgi = (32 + vector) as u32;
        let mut sgi_routing = get_kvm_sgi_routing_instance().lock().unwrap();
        sgi_routing.add_msi_gsi_routing(sgi, entry.addr_lo, entry.addr_hi, entry.data);
        drop(sgi_routing);
        self.vm.set_gsi_routing().unwrap();

        if entry.is_mask() {
            self.vm
                .del_irqfd(&msix_info.event_fds[vector], sgi)
                .unwrap();
        } else {
            self.vm
                .set_irqfd(&msix_info.event_fds[vector], sgi)
                .unwrap();
        }
    }

    fn read_msix_pba(&self, _offset: u64, _buf: &mut [u8]) {
        todo!()
    }

    fn write_msix_pba(&self, _offset: u64, _buf: &[u8]) {
        todo!()
    }

    fn enable_intx(&self) {
        let Some(intx_info) = &self.interrupt_info.intx else {
            return;
        };

        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(intx) = &mut interrupt_manager.intx else {
            return;
        };

        if !intx.enabled {
            self.device.enable_intx(&intx_info.trigger_fd).unwrap();
            self.device
                .set_intx_resample_fd(&intx_info.resample_fd)
                .unwrap();
            intx.enabled = true;
        }
    }

    fn disable_intx(&self) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(intx) = &mut interrupt_manager.intx else {
            return;
        };

        if intx.enabled {
            self.device.disable_intx().unwrap();
            intx.enabled = false;
        }
    }

    fn enable_msix(&self) {
        let Some(msix_info) = &self.interrupt_info.msix else {
            return;
        };

        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(msix) = &mut interrupt_manager.msix else {
            return;
        };

        if !msix.enabled {
            self.device
                .enable_msix(msix_info.event_fds.iter().collect())
                .unwrap();
            msix.enabled = true;
        }
    }

    fn disable_msix(&self) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(msix) = &mut interrupt_manager.msix else {
            return;
        };

        if msix.enabled {
            self.device.disable_msix().unwrap();
            msix.enabled = false;
        }
    }

    fn on_msix_ctrl_changing(&self, old_ctrl: u16, new_ctrl: u16) {
        // enable msix
        if old_ctrl & PCI_MSIX_FLAGS_ENABLE == 0 && new_ctrl & PCI_MSIX_FLAGS_ENABLE != 0 {
            self.disable_intx();
            self.enable_msix();
        }

        // disable msix
        if old_ctrl & PCI_MSIX_FLAGS_ENABLE != 0 && new_ctrl & PCI_MSIX_FLAGS_ENABLE == 0 {
            self.disable_msix();
            self.enable_intx();
        }
    }

    fn write_capability(&self, offset: u16, buf: &[u8]) {
        let mut configuration_space = self.configuration_space.lock().unwrap();

        if let Some(msix_info) = &self.interrupt_info.msix
            && msix_info.cap_offset_range.contains(&offset)
        {
            let cap = PciMsixCap::mut_from_bytes(
                &mut configuration_space.as_bytes_mut()[msix_info.cap_offset_range.start as usize
                    ..msix_info.cap_offset_range.end as usize],
            )
            .unwrap();

            let offset_within_cap = offset - msix_info.cap_offset_range.start;

            // ctrl
            if offset_within_cap == PciMsixCapOffset::Ctrl as u16 {
                let old_ctrl = cap.ctrl;
                let new_ctrl = u16::from_le_bytes(buf.try_into().unwrap());
                cap.ctrl = new_ctrl;
                self.on_msix_ctrl_changing(old_ctrl, new_ctrl);
            } else {
                panic!("guest try to write a invalid field of msi-x cap")
            }
        }

        if let Some(msi) = &self.interrupt_info.msi
            && msi.cap_offset_range.contains(&offset)
        {
            todo!()
        }
    }

    fn write_bar(&self, bar_index: usize, buf: &[u8]) {
        let mut configuration_space = self.configuration_space.lock().unwrap();
        let header = configuration_space.as_header_mut::<Type0Header>();

        let data = u32::from_le_bytes(buf.try_into().unwrap());

        if data == u32::MAX {
            let size = if let Some(bar_info) = &self.bars[bar_index] {
                let len: usize = match bar_info {
                    #[cfg(target_arch = "x86_64")]
                    PciBarInfo::Pio { len } => *len,
                    PciBarInfo::Mmio { len, .. } => *len,
                };
                len.try_into().unwrap()
            } else if bar_index > 0
                && let Some(bar_info) = &self.bars[bar_index - 1]
                && let PciBarInfo::Mmio {
                    is_64bit: true,
                    len,
                } = bar_info
            {
                (*len >> 32) as u32
            } else {
                0
            };

            header.bar[bar_index] = !(size.wrapping_sub(1));
        } else {
            header.bar[bar_index] = data;
        }
    }

    fn write_command(&self, command: u16) -> Option<EcamUpdateCallback> {
        let mut callback_ops = vec![];

        let mut configuration_space = self.configuration_space.lock().unwrap();

        let header = configuration_space.as_header_mut::<Type0Header>();
        let old_command = header.common.command;
        header.common.command = command;

        let old_command = PciCommand::from_bits_retain(old_command);
        let command = PciCommand::from_bits_retain(command);

        for (i, bar) in self.bars.iter().enumerate() {
            let Some(bar_info) = bar else {
                continue;
            };

            let bar = header.bar[i];

            match bar_info {
                #[cfg(target_arch = "x86_64")]
                PciBarInfo::Pio { len } => {
                    let address = address_of_bar(bar);

                    if command.contains(PciCommand::IO) && !old_command.contains(PciCommand::IO) {
                        callback_ops.push(EcamUpdateCallbackOps::AddPioRouter {
                            bar: i as u8,
                            port: address as u16..address as u16 + *len as u16,
                        });
                    } else if !command.contains(PciCommand::IO)
                        && old_command.contains(PciCommand::IO)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::RemovePioRouter { bar: i as u8 });
                    }
                }
                PciBarInfo::Mmio { is_64bit, len } => {
                    let address = if *is_64bit {
                        (header.bar[i + 1] as u64) << 32 | address_of_bar(bar) as u64
                    } else {
                        address_of_bar(bar) as u64
                    };
                    if command.contains(PciCommand::MEMORY)
                        && !old_command.contains(PciCommand::MEMORY)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::AddMmioRouter {
                            bar: i as u8,
                            pci_address_range: address..address + *len as u64,
                        });
                    } else if !command.contains(PciCommand::MEMORY)
                        && old_command.contains(PciCommand::MEMORY)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::RemoveMmioRouter { bar: i as u8 });
                    }
                }
            }
        }

        Some(EcamUpdateCallback(callback_ops))
    }
}

impl PciFunctionArch for VfioPciFunction {
    fn interrupt_map_entry(
        &self,
        _bus: u8,
        _device: u8,
        _function: u8,
    ) -> Option<InterruptMapEntry> {
        todo!()
    }
}

impl PciFunction for VfioPciFunction {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        if offset < CommonHeaderOffset::CapabilityStart as u16 {
            let Some(field) = Type0HeaderOffset::from_repr(offset) else {
                warn!(?offset, "reading from a invalid header field");
                panic!()
            };

            match field {
                Type0HeaderOffset::VendorId
                | Type0HeaderOffset::DeviceId
                | Type0HeaderOffset::RevisionId
                | Type0HeaderOffset::ProgIf
                | Type0HeaderOffset::Subclass
                | Type0HeaderOffset::ClassCode
                | Type0HeaderOffset::CacheLineSize
                | Type0HeaderOffset::LatencyTimer
                | Type0HeaderOffset::HeaderType
                | Type0HeaderOffset::Bist
                | Type0HeaderOffset::CardbusCisPointer
                | Type0HeaderOffset::SubsystemVendorId
                | Type0HeaderOffset::SubsystemId
                | Type0HeaderOffset::RomBaseAddress
                | Type0HeaderOffset::Reserved
                | Type0HeaderOffset::InterruptLine
                | Type0HeaderOffset::InterruptPin
                | Type0HeaderOffset::MinGnt
                | Type0HeaderOffset::MaxLat => {
                    buf.copy_from_slice(
                        &self.raw_configuration_space.as_bytes()
                            [offset as usize..offset as usize + buf.len()],
                    );
                }
                Type0HeaderOffset::Command | Type0HeaderOffset::Status => {
                    self.device
                        .region_read(VFIO_PCI_CONFIG_REGION_INDEX, buf, offset as u64)
                        .unwrap();
                }
                Type0HeaderOffset::Bar0
                | Type0HeaderOffset::Bar1
                | Type0HeaderOffset::Bar2
                | Type0HeaderOffset::Bar3
                | Type0HeaderOffset::Bar4
                | Type0HeaderOffset::Bar5
                | Type0HeaderOffset::CapPointer => {
                    self.configuration_space.lock().unwrap().read(offset, buf)
                }
            }
        } else {
            self.configuration_space.lock().unwrap().read(offset, buf);
        }
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback> {
        if offset < CommonHeaderOffset::CapabilityStart as u16 {
            let Some(field) = Type0HeaderOffset::from_repr(offset) else {
                warn!(?offset, "writing to a invalid header field");
                panic!()
            };

            match field {
                Type0HeaderOffset::VendorId
                | Type0HeaderOffset::DeviceId
                | Type0HeaderOffset::RevisionId
                | Type0HeaderOffset::ProgIf
                | Type0HeaderOffset::Subclass
                | Type0HeaderOffset::ClassCode
                | Type0HeaderOffset::CacheLineSize
                | Type0HeaderOffset::LatencyTimer
                | Type0HeaderOffset::HeaderType
                | Type0HeaderOffset::Bist
                | Type0HeaderOffset::CardbusCisPointer
                | Type0HeaderOffset::SubsystemVendorId
                | Type0HeaderOffset::SubsystemId
                | Type0HeaderOffset::RomBaseAddress
                | Type0HeaderOffset::CapPointer
                | Type0HeaderOffset::Reserved
                | Type0HeaderOffset::InterruptLine
                | Type0HeaderOffset::InterruptPin
                | Type0HeaderOffset::MinGnt
                | Type0HeaderOffset::MaxLat => {
                    let mut configuration_space = self.configuration_space.lock().unwrap();
                    configuration_space.as_bytes_mut()
                        [offset as usize..offset as usize + buf.len()]
                        .copy_from_slice(buf);
                }
                Type0HeaderOffset::Command => {
                    let cb = self.write_command(u16::from_le_bytes(buf.try_into().unwrap()));
                    self.device
                        .region_write(VFIO_PCI_CONFIG_REGION_INDEX, buf, offset as u64)
                        .unwrap();
                    return cb;
                }
                Type0HeaderOffset::Status => {
                    self.device
                        .region_write(VFIO_PCI_CONFIG_REGION_INDEX, buf, offset as u64)
                        .unwrap();
                }
                Type0HeaderOffset::Bar0 => self.write_bar(0, buf),
                Type0HeaderOffset::Bar1 => self.write_bar(1, buf),
                Type0HeaderOffset::Bar2 => self.write_bar(2, buf),
                Type0HeaderOffset::Bar3 => self.write_bar(3, buf),
                Type0HeaderOffset::Bar4 => self.write_bar(4, buf),
                Type0HeaderOffset::Bar5 => self.write_bar(5, buf),
            }

            None
        } else {
            self.write_capability(offset, buf);

            None
        }
    }

    fn bar_read(&self, bar: u8, offset: u64, buf: &mut [u8]) {
        self.device
            .region_read(VFIO_PCI_BAR0_REGION_INDEX + bar as u32, buf, offset)
            .unwrap();

        if let Some(msix) = &self.interrupt_info.msix {
            let table_range =
                msix.table_offset as u64..msix.table_offset as u64 + msix.table_len as u64;
            let pba_range = msix.pba_offset as u64..msix.pba_offset as u64 + msix.pba_len as u64;

            if msix.table_bar == bar && table_range.contains(&offset) {
                self.read_msix_table((offset - table_range.start).try_into().unwrap(), buf);
            }

            if msix.pba_bar == bar && pba_range.contains(&offset) {
                self.read_msix_pba((offset - pba_range.start).try_into().unwrap(), buf);
            }
        }
    }

    fn bar_write(&self, bar: u8, offset: u64, buf: &[u8]) {
        self.device
            .region_write(VFIO_PCI_BAR0_REGION_INDEX + bar as u32, buf, offset)
            .unwrap();

        if let Some(msix) = &self.interrupt_info.msix {
            let table_range =
                msix.table_offset as u64..msix.table_offset as u64 + msix.table_len as u64;
            let pba_range = msix.pba_offset as u64..msix.pba_offset as u64 + msix.pba_len as u64;

            if msix.table_bar == bar && table_range.contains(&offset) {
                self.write_msix_table(msix, (offset - table_range.start).try_into().unwrap(), buf);
            }

            if msix.pba_bar == bar && pba_range.contains(&offset) {
                self.write_msix_pba((offset - pba_range.start).try_into().unwrap(), buf);
            }
        }
    }

    fn legacy_irq(&self) -> Option<(u8, u8)> {
        self.interrupt_info
            .intx
            .as_ref()
            .map(|intx| (intx.line, intx.pin as u8))
    }
}
