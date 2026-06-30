use std::sync::Arc;
use std::sync::Mutex;

use tracing::warn;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_CONFIG_REGION_INDEX;
use vm_core::interrupt_manager::InterruptManager;
use vm_core::virtualization::kvm::gsi_routing::get_kvm_gsi_routing_instance;
use vm_core::virtualization::vm::HypervisorVm;
use vm_pci::device::capability::msi::PciMsiCap;
use vm_pci::device::capability::msi::PciMsiCap64;
use vm_pci::device::capability::msi::PciMsiCap64Mask;
use vm_pci::device::capability::msi::PciMsiCapMask;
use vm_pci::device::capability::msi::PciMsiCapOps;
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
use crate::vfio_pci::interrupt::msi::VfioMsi;
use crate::vfio_pci::interrupt::msi::VfioMsiInfo;
use crate::vfio_pci::interrupt::msix::VfioMsixInfo;

pub struct VfioPciFunction {
    vm: Arc<dyn HypervisorVm>,
    irq_manager: Arc<InterruptManager>,
    raw_configuration_space: PciConfigurationSpace,
    configuration_space: Mutex<ConfigurationSpace>,
    bars: [Option<PciBarInfo>; 6],
    device: Arc<VfioDevice>,
    interrupt_manager: Arc<Mutex<VfioInterruptManager>>,
    interrupt_info: VfioInterruptInfo,
}

impl VfioPciFunction {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        vm: Arc<dyn HypervisorVm>,
        irq_manager: Arc<InterruptManager>,
        raw_configuration_space: PciConfigurationSpace,
        configuration_space: ConfigurationSpace,
        bars: [Option<PciBarInfo>; 6],
        device: Arc<VfioDevice>,
        interrupt_info: VfioInterruptInfo,
        interrupt_manager: VfioInterruptManager,
    ) -> Self {
        VfioPciFunction {
            vm,
            irq_manager,
            raw_configuration_space,
            configuration_space: configuration_space.into(),
            bars,
            device,
            interrupt_info,
            interrupt_manager: Arc::new(Mutex::new(interrupt_manager)),
        }
    }

    fn insert_or_update_msi_gsi_entry(&self, addr_lo: u32, addr_hi: u32, data: u32, gsi: u32) {
        let updated = {
            let mut gsi_routing = get_kvm_gsi_routing_instance().lock().unwrap();
            gsi_routing.insert_or_update_msi_gsi_routing(gsi, addr_lo, addr_hi, data)
        };

        if updated {
            self.vm.set_gsi_routing().unwrap();
        }
    }
}

impl VfioPciFunction {
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
            self.vm
                .set_irqfd_with_resample(
                    &intx_info.trigger_fd,
                    &intx_info.resample_fd,
                    intx_info.gsi,
                )
                .unwrap();
            intx.enabled = true;
        }
    }

    fn disable_intx(&self) {
        let Some(intx_info) = &self.interrupt_info.intx else {
            return;
        };

        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(intx) = &mut interrupt_manager.intx else {
            return;
        };

        if intx.enabled {
            self.device.disable_intx().unwrap();
            self.vm
                .del_irqfd(&intx_info.trigger_fd, intx_info.gsi)
                .unwrap();
            self.vm
                .del_irqfd(&intx_info.resample_fd, intx_info.gsi)
                .unwrap();
            intx.enabled = false;
        }
    }
}

impl VfioPciFunction {
    fn update_msi_vector_enable(
        &self,
        msi_info: &VfioMsiInfo,
        msi: &mut VfioMsi,
        msi_cap: &dyn PciMsiCapOps,
        enable: bool,
        vector: usize,
    ) {
        let gsi = if let Some(gsi) = msi.gsi[vector] {
            gsi
        } else {
            let gsi = self.irq_manager.allocate_gsi().unwrap();
            msi.gsi[vector] = Some(gsi);
            gsi
        };

        self.insert_or_update_msi_gsi_entry(
            msi_cap.address_lo(),
            msi_cap.address_hi(),
            msi_cap.vector_data(vector),
            gsi,
        );
        let fd = &msi_info.event_fds[vector];
        if enable {
            self.vm.set_irqfd(fd, gsi).unwrap();
            msi.irqrd[vector] = true;
        } else {
            self.vm.del_irqfd(fd, gsi).unwrap();
            msi.irqrd[vector] = false;
        }
    }

    fn update_msi_vector_routing(
        &self,
        old_msi_cap: &dyn PciMsiCapOps,
        new_msi_cap: &dyn PciMsiCapOps,
        vector: usize,
    ) {
        let Some(msi_info) = &self.interrupt_info.msi else {
            return;
        };

        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(msi) = &mut interrupt_manager.msi else {
            return;
        };

        if old_msi_cap.is_mask(vector) != new_msi_cap.is_mask(vector) {
            self.update_msi_vector_enable(
                msi_info,
                msi,
                new_msi_cap,
                !new_msi_cap.is_mask(vector),
                vector,
            );
        }
    }

    fn force_update_msi_vector_routing(
        &self,
        msi_info: &VfioMsiInfo,
        msi: &mut VfioMsi,
        msi_cap: &dyn PciMsiCapOps,
    ) {
        if msi_cap.mask_bits_offset().is_some() {
            return;
        }

        for vector in 0..msi_cap.configured_vectors() {
            if !msi.irqrd[vector] && msi_cap.enable() {
                self.update_msi_vector_enable(msi_info, msi, msi_cap, true, vector);
            }

            if msi.irqrd[vector] && !msi_cap.enable() {
                self.update_msi_vector_enable(msi_info, msi, msi_cap, false, vector);
            }
        }
    }

    fn enable_msi(&self, msi_info: &VfioMsiInfo, new_msi_cap: &dyn PciMsiCapOps) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(msi) = &mut interrupt_manager.msi else {
            return;
        };

        if !msi.enabled {
            self.force_update_msi_vector_routing(msi_info, msi, new_msi_cap);
            self.device
                .enable_msi(msi_info.event_fds.iter().collect())
                .unwrap();
            msi.enabled = true;
        }
    }

    fn disable_msi(&self, msi_info: &VfioMsiInfo, new_msi_cap: &dyn PciMsiCapOps) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let Some(msi) = &mut interrupt_manager.msi else {
            return;
        };

        if msi.enabled {
            self.force_update_msi_vector_routing(msi_info, msi, new_msi_cap);
            self.device.disable_msi().unwrap();
            msi.enabled = false;
        }
    }

    fn on_msi_mask_bits_changing(
        &self,
        old_msi_cap: &dyn PciMsiCapOps,
        msi_cap: &dyn PciMsiCapOps,
    ) {
        for vector in 0..msi_cap.available_vectors() {
            self.update_msi_vector_routing(old_msi_cap, msi_cap, vector);
        }
    }

    fn on_msi_ctrl_enable_changing(&self, new_msi_cap: &dyn PciMsiCapOps) {
        let Some(msi_info) = &self.interrupt_info.msi else {
            return;
        };

        if new_msi_cap.enable() {
            self.disable_intx();
            self.enable_msi(msi_info, new_msi_cap);
        } else {
            self.disable_msi(msi_info, new_msi_cap);
            self.enable_intx();
        }
    }

    fn sync_msi_cap_changing(&self, old_cap: &dyn PciMsiCapOps, new_cap: &dyn PciMsiCapOps) {
        if old_cap.mask_bits() != new_cap.mask_bits() {
            self.on_msi_mask_bits_changing(old_cap, new_cap);
        }

        if old_cap.enable() != new_cap.enable() {
            self.on_msi_ctrl_enable_changing(new_cap);
        }
    }

    fn update_msi_capability(&self, msi_info: &VfioMsiInfo, offset_within_cap: u16, buf: &[u8]) {
        let mut configuration_space = self.configuration_space.lock().unwrap();

        let msi_cap_buf = &mut configuration_space.as_bytes_mut()
            [msi_info.cap_offset_range.start as usize..msi_info.cap_offset_range.end as usize];

        // Skip `cap_id` and `next`, since the length of msi cap is variable
        let old_cap: Box<dyn PciMsiCapOps> = match (msi_info.bit64, msi_info.mask) {
            (false, false) => Box::new(PciMsiCap::read_from_bytes(&msi_cap_buf[2..]).unwrap()),
            (true, false) => Box::new(PciMsiCap64::read_from_bytes(&msi_cap_buf[2..]).unwrap()),
            (false, true) => Box::new(PciMsiCapMask::read_from_bytes(&msi_cap_buf[2..]).unwrap()),
            (true, true) => Box::new(PciMsiCap64Mask::read_from_bytes(&msi_cap_buf[2..]).unwrap()),
        };

        msi_cap_buf[offset_within_cap as usize..offset_within_cap as usize + buf.len()]
            .copy_from_slice(buf);
        let cap: &dyn PciMsiCapOps = match (msi_info.bit64, msi_info.mask) {
            (false, false) => PciMsiCap::ref_from_bytes(&msi_cap_buf[2..]).unwrap(),
            (true, false) => PciMsiCap64::ref_from_bytes(&msi_cap_buf[2..]).unwrap(),
            (false, true) => PciMsiCapMask::ref_from_bytes(&msi_cap_buf[2..]).unwrap(),
            (true, true) => PciMsiCap64Mask::ref_from_bytes(&msi_cap_buf[2..]).unwrap(),
        };

        self.sync_msi_cap_changing(old_cap.as_ref(), cap);
    }
}

impl VfioPciFunction {
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

    fn read_msix_table(&self, offset: usize, buf: &mut [u8]) {
        let interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_ref().unwrap();

        buf.copy_from_slice(&msix.table.as_bytes()[offset..offset + buf.len()]);
    }

    fn write_msix_table(&self, msix_info: &VfioMsixInfo, offset: usize, buf: &[u8]) {
        // ensure field writing
        let _val = u32::from_le_bytes(buf.try_into().unwrap());

        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_mut().unwrap();

        let vector = offset / size_of::<MsixEntry>();
        let offset_within_entry = offset % size_of::<MsixEntry>();

        let msi_entry_old = msix.table[vector].clone();
        let msi_entry_new = &mut msix.table[vector];
        msi_entry_new.as_mut_bytes()[offset_within_entry..offset_within_entry + buf.len()]
            .copy_from_slice(buf);

        let gsi = if let Some(gsi) = msix.gsi[vector] {
            gsi
        } else {
            let gsi = self.irq_manager.allocate_gsi().unwrap();
            msix.gsi[vector] = Some(gsi);
            gsi
        };
        self.insert_or_update_msi_gsi_entry(
            msi_entry_new.addr_lo,
            msi_entry_new.addr_hi,
            msi_entry_new.data,
            gsi,
        );
        if msi_entry_old.is_mask() != msi_entry_new.is_mask() {
            if msi_entry_new.is_mask() {
                self.vm
                    .del_irqfd(&msix_info.event_fds[vector], gsi)
                    .unwrap();
            } else {
                self.vm
                    .set_irqfd(&msix_info.event_fds[vector], gsi)
                    .unwrap();
            }
        }
    }

    fn read_msix_pba(&self, offset: usize, buf: &mut [u8]) {
        let interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_ref().unwrap();

        buf.copy_from_slice(&msix.pba[offset..offset + buf.len()]);
    }

    fn write_msix_pba(&self, offset: usize, buf: &[u8]) {
        let mut interrupt_manager = self.interrupt_manager.lock().unwrap();
        let msix = interrupt_manager.msix.as_mut().unwrap();

        msix.pba[offset..offset + buf.len()].copy_from_slice(buf);
    }

    fn on_msix_ctrl_changing(&self, cap: &mut PciMsixCap, new_ctrl: u16) {
        let old_ctrl = cap.ctrl;
        cap.ctrl = new_ctrl;

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

    fn update_msix_capability(&self, msix_info: &VfioMsixInfo, offset_within_cap: u16, buf: &[u8]) {
        let mut configuration_space = self.configuration_space.lock().unwrap();

        let cap = PciMsixCap::mut_from_bytes(
            &mut configuration_space.as_bytes_mut()[msix_info.cap_offset_range.start as usize
                ..msix_info.cap_offset_range.end as usize],
        )
        .unwrap();

        if offset_within_cap == PciMsixCapOffset::Ctrl as u16 {
            self.on_msix_ctrl_changing(cap, u16::from_le_bytes(buf.try_into().unwrap()));
        } else {
            warn!(
                offset_within_cap,
                "guest try to write a invalid field of msi-x cap"
            )
        }
    }
}

impl VfioPciFunction {
    fn write_capability(&self, offset: u16, buf: &[u8]) {
        if let Some(msi_info) = &self.interrupt_info.msi
            && msi_info.cap_offset_range.contains(&offset)
        {
            self.update_msi_capability(msi_info, offset - msi_info.cap_offset_range.start, buf);
        }

        if let Some(msix_info) = &self.interrupt_info.msix
            && msix_info.cap_offset_range.contains(&offset)
        {
            self.update_msix_capability(msix_info, offset - msix_info.cap_offset_range.start, buf);
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
