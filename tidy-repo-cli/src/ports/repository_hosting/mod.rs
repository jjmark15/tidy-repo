pub use ports::*;

pub mod adapters;
mod ports;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AuthenticationCredentialValidity {
    Valid,
    Invalid,
}
