pub mod adapters;
pub mod github;

#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationCredentialValidity {
    Valid,
    Invalid,
}
