use std::iter;

use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_BAR5_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_CONFIG_REGION_INDEX;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_READ;
use vfio_bindings::bindings::vfio::VFIO_REGION_INFO_FLAG_WRITE;
use vm_core::device::Device;
use vm_pci::device::function::type0::Type0Function;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;
use zerocopy::FromBytes;

use crate::error::Error;
use crate::error::Result;
use crate::vfio::device::VfioDevice;
use crate::vfio_pci::function::VfioBarInfo;
use crate::vfio_pci::function::VfioPciFunction;

pub struct VfioPciDevice {
    name: String,
    _vfio_device: VfioDevice,
    function: Type0Function<VfioPciFunction>,
}

impl VfioPciDevice {
    pub fn new(name: String, vfio_device: VfioDevice) -> Result<Self> {
        vfio_device.reset()?;

        let header = {
            let pci_config_region = vfio_device
                .get_region_info(VFIO_PCI_CONFIG_REGION_INDEX)
                .ok_or(Error::NoPciConfigRegion)?;

            let mut buf = vec![0; pci_config_region.size as usize];
            vfio_device.region_read(VFIO_PCI_CONFIG_REGION_INDEX, &mut buf, 0)?;
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_READ != 0);
            assert!(pci_config_region.flags & VFIO_REGION_INFO_FLAG_WRITE != 0);

            let header = Type0Header::read_from_bytes(&mut buf[..size_of::<Type0Header>()])
                .map_err(|_| Error::PciHeader)?;

            header
        };

        let mut bar_info = [const { None }; 6];
        {
            for region in VFIO_PCI_BAR0_REGION_INDEX..=VFIO_PCI_BAR5_REGION_INDEX {
                if let Some(pci_config_region) = vfio_device.get_region_info(region) {
                    bar_info[region as usize] = Some(VfioBarInfo {
                        size: pci_config_region.size,
                    });
                }
            }
        }

        let function = Type0Function::new(VfioPciFunction::new(header, bar_info))?;

        Ok(VfioPciDevice {
            name,
            _vfio_device: vfio_device,
            function,
        })
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
