use std::sync::Arc;

use crate::device::virtqueue::VirtioConfigurationChangeNotifier;

pub mod mmio;
pub mod pci;

pub(crate) mod common;

pub trait VirtioDeviceOps {
    fn configuration_change_notifier(&self) -> Arc<dyn VirtioConfigurationChangeNotifier>;
}
