#[cfg(feature = "oidc")]
pub mod oidc;

pub struct Identity {
    pub username: String,
    pub groups: Vec<String>,
}

#[cfg(not(feature = "oidc"))]
pub mod oidc {
    use super::Identity;
    use serde::Deserialize;

    type AuthError = std::io::Error;

    #[derive(Deserialize, Default, Clone, Debug)]
    pub struct OidcAuthCfg;

    #[derive(Default, Clone)]
    pub struct OidcClient {
        pub authorize_url: String,
    }

    impl OidcClient {
        pub async fn from_config(_cfg: &OidcAuthCfg) -> Self {
            Self::default()
        }
    }

    #[derive(Deserialize)]
    pub struct AuthRequest;

    impl AuthRequest {
        pub async fn auth(&self, _oidc: &OidcClient) -> Result<Identity, AuthError> {
            unimplemented!()
        }
    }
}
