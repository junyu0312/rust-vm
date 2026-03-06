pub enum NotificationEvent {
    ConfigurationChange,
    AvailableBuffer,
}

pub trait TransportContext {
    fn update_config_generation_and_notify(&self);
}
