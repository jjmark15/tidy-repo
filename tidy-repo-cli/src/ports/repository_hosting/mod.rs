pub use ports::*;

pub mod adapters;
mod ports;

#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationCredentialValidity {
    Valid,
    Invalid,
}
