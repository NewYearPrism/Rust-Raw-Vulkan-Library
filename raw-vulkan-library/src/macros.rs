#[macro_export]
macro_rules! call_conv {
    (unsafe fn $tok:tt) => {unsafe extern "system" fn $tok};
    (fn $tok:tt) => {unsafe extern "system" fn $tok};
}
