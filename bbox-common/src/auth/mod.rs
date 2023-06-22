pub mod oidc;

pub struct Identity {
    pub username: String,
    pub groups: Vec<String>,
}
