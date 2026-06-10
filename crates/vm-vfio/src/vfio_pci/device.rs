use std::iter;

use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR5_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_CONFIG_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_READ;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_WRITE;
use vm_core::device::Device;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_32;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_64;
use vm_pci::types::bar::PCI_BASE_ADDRESS_MEM_TYPE_MASK;
use vm_pci::types::bar::PCI_BASE_ADDRESS_SPACE;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;

use crate::error::Error;
use crate::error::Result;
use crate::vfio::device::VfioDevice;
use crate::vfio_pci::function::VfioBarInfo;
use crate::vfio_pci::function::VfioBarResource;
use crate::vfio_pci::function::VfioPciFunction;

pub struct VfioPciDevice {
    name: String,
    function: VfioPciFunction,
}

impl VfioPciDevice {
    pub fn new(name: String, vfio_device: VfioDevice) -> Result<Self> {
        vfio_device.reset()?;

        let mut configuration_space = ConfigurationSpace::default();

        // Copy header from device
        {
            let pci_config_region = vfio_device.get_region_info(VFIO_PCI_CONFIG_REGION_INDEX)?;
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_READ != 0);
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_WRITE != 0);

            let mut buf = vec![0; pci_config_region.size as usize];
            vfio_device.region_read(VFIO_PCI_CONFIG_REGION_INDEX, &mut buf, 0)?;

            configuration_space.as_bytes_mut()[0..buf.len()].copy_from_slice(&buf);
        }

        // Prepare header
        {
            let header = configuration_space.as_header_mut::<Type0Header>();

            if header.common.header_type != 0 {
                return Err(Error::VfioPciDeviceIsNotEndpoint);
            }

            // TODO: Should we emulate irq_line?
            // TODO: Should we reconstruct cap?
            // TODO: Should we emulate status?
        };

        let header = configuration_space.as_header::<Type0Header>();

        let mut bar_info = [const { None }; 6];
        {
            for index in VFIO_PCI_BAR0_REGION_INDEX..=VFIO_PCI_BAR5_REGION_INDEX {
                let region = vfio_device.get_region_info(index)?;

                if region.size == 0 {
                    continue;
                }

                let index = index as usize;
                let bar = header.bar[index];

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

                    VfioBarResource::Mmio { is_64bit }
                } else {
                    VfioBarResource::Pio
                };

                bar_info[index] = Some(VfioBarInfo {
                    size: region.size,
                    // TODO: We should alloc resource from vmm/pci mananger
                    resource,
                });
            }
        }

        let function = VfioPciFunction::new(configuration_space, bar_info);

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
