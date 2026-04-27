use std::collections::HashMap;

use vm_core::monitor::MonitorCommandOps;
use vm_core::monitor::MonitorError;

#[derive(Default)]
pub struct MonitorServerBuilder {
    pub components: HashMap<String, Box<dyn MonitorCommandOps>>,
}

impl MonitorServerBuilder {
    pub fn register_command_handler(
        &mut self,
        name: &str,
        handler: Box<dyn MonitorCommandOps>,
    ) -> Result<(), MonitorError> {
        let name = name.to_string();

        if self.components.contains_key(&name) {
            return Err(MonitorError::CommandHandlerConflict(name));
        }

        self.components.insert(name, handler);

        Ok(())
    }
}
