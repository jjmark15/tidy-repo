pub mod github;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AuthenticationCredentialValidity {
    Valid,
    Invalid,
}
