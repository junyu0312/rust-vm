#[allow(non_camel_case_types)]
pub enum PsciRet {
    SUCCESS = 0,
    NOT_SUPPORTED = -1,
    INVALID_PARAMS = -2,
    DENIED = -3,
    ALREADY_ON = -4,
    ON_PENDING = -5,
    INTERNAL_FAILURE = -6,
    NOT_PRESENT = -7,
    DISABLED = -8,
    INVALID_ADDRESS = -9,
}
