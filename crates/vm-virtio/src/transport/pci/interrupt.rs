use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use vm_core::arch::irq::InterruptController;
use vm_pci::device::capability::msix::PCI_MSIX_FLAGS_ENABLE;
use vm_pci::device::capability::msix::PciMsixCap;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::command::PciCommand;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::configuration_space::status::PciStatus;
use zerocopy::FromBytes;

use crate::device::virtqueue::VirtioConfigurationChangeNotifier;
use crate::device::virtqueue::VirtioUsedBufferNotifier;
use crate::transport::pci::VirtioPciMsixVector;
use crate::transport::pci::msix::VirtioPciMsixInfo;
use crate::types::interrupt_status::InterruptStatus;

pub struct VirtioPciIrqDispatcher {
    pub irq_chip: Arc<dyn InterruptController>,
    pub configuration_space: Arc<Mutex<ConfigurationSpace>>,
    pub virtio_pci_msix_vector: RwLock<VirtioPciMsixVector>,
    pub legacy_int: Option<u8>,
    pub msix: Option<Arc<RwLock<VirtioPciMsixInfo>>>,
    // TODO: ugly, it is actually a const value
    pub msix_cap_offset: Mutex<Option<u8>>,
}

impl VirtioPciIrqDispatcher {
    fn notify_used_buffer(&self, queue_sel: u16) {
        if self.msix_enabled() {
            let vector = self
                .virtio_pci_msix_vector
                .read()
                .unwrap()
                .queue_msix_vector[queue_sel as usize];
            let msix = self.msix.as_ref().unwrap();
            let msi = msix.read().unwrap();
            let msi = &msi.table[vector as usize];
            self.irq_chip.send_msi(msi.addr_lo, msi.addr_hi, msi.data);
        } else {
            // If MSI-X capability is disabled, the device MUST set the Interrupt Status bit in the PCI Status register in the
            // PCI Configuration Header of the device to the logical OR of all bits in ISR status of the device. The device
            // then asserts/deasserts INT#x interrupts unless masked according to standard PCI rules [PCI].
            let mut cfg = self.configuration_space.lock().unwrap();
            let header = cfg.as_header_mut::<Type0Header>();
            if !PciCommand::from_bits_retain(header.common.command)
                .contains(PciCommand::INTX_DISABLE)
            {
                header.common.status |= PciStatus::Interrupt as u16;
                let irq = self.legacy_int.as_ref().unwrap();
                self.irq_chip.trigger_irq(*irq as u32, true);
            }
        }
    }

    fn notify_configuration_change(&self) {
        if self.msix_enabled() {
            let vector = self
                .virtio_pci_msix_vector
                .read()
                .unwrap()
                .config_msix_vector;
            let msix = self.msix.as_ref().unwrap();
            let msi = msix.read().unwrap();
            let msi = &msi.table[vector as usize];
            self.irq_chip.send_msi(msi.addr_lo, msi.addr_hi, msi.data);
        }
    }

    fn msix_enabled(&self) -> bool {
        if let Some(msix_cap_offset) = *self.msix_cap_offset.lock().unwrap() {
            let msix_cap_offset = msix_cap_offset as usize;

            let cfg = self.configuration_space.lock().unwrap();

            let cap = PciMsixCap::ref_from_bytes(
                &cfg.as_bytes()[msix_cap_offset..msix_cap_offset + size_of::<PciMsixCap>()],
            )
            .unwrap();

            cap.ctrl & PCI_MSIX_FLAGS_ENABLE != 0
        } else {
            false
        }
    }
}

pub struct VirtioPciEventUsedBufferNotifier {
    pub interrupt_dispatcher: Arc<VirtioPciIrqDispatcher>,
    pub queue_sel: u16,
    pub is: Arc<Mutex<InterruptStatus>>,
}

impl VirtioUsedBufferNotifier for VirtioPciEventUsedBufferNotifier {
    fn notify_used_buffer(&self) {
        if !self.interrupt_dispatcher.msix_enabled() {
            // If MSI-X capability is disabled, the device MUST set the Queue Interrupt bit in ISR status before sending a
            // virtqueue notification to the driver.
            self.is
                .lock()
                .unwrap()
                .insert(InterruptStatus::VIRTIO_MMIO_INT_VRING);
        }

        self.interrupt_dispatcher.notify_used_buffer(self.queue_sel);
    }
}

pub struct VirtioPciConfigurationChangeNotifier {
    pub interrupt_dispatcher: Arc<VirtioPciIrqDispatcher>,
    pub is: Arc<Mutex<InterruptStatus>>,
    pub config_generation: Arc<Mutex<u8>>,
}

impl VirtioConfigurationChangeNotifier for VirtioPciConfigurationChangeNotifier {
    fn update_config_generation(&self) {
        // The device MUST set the Device Configuration Interrupt bit in ISR status before sending a device configu-
        // ration change notification to the driver.
        *self.config_generation.lock().unwrap() += 1;
        self.is
            .lock()
            .unwrap()
            .insert(InterruptStatus::VIRTIO_MMIO_INT_CONFIG);

        self.interrupt_dispatcher.notify_configuration_change();
    }
}
