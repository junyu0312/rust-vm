/*
 * Synchronization events.
 */

#[repr(u16)]
pub enum EvSynCode {
    Report = 0,
    Config = 1,
    MtReport = 2,
    Dropped = 3,
}
