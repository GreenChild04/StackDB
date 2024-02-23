pub mod base;
pub mod errors;

pub enum Or<A, B> {
    A(A),
    B(B),
}
