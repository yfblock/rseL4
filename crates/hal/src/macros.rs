#[macro_export]
macro_rules! bit {
    ($idx:expr) => {
        (1 << ($idx))
    };
}
