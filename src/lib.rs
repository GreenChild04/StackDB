pub mod base;
pub mod errors;
pub mod prelude;
pub mod default;

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn check_iter_val<T: std::fmt::Debug>(value: T) -> T {
    dbg!(&value);
    value
}
