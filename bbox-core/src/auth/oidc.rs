use super::Identity;
use log::{debug, info};
use openidconnect::{
    core::{CoreClient, CoreErrorResponseType, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClaimsVerificationError, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, RequestTokenError, Scope,
    StandardErrorResponse,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    OidcRequestTokenError(
        #[from]
        RequestTokenError<
            openidconnect::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),
    #[error(transparent)]
    OidcClaimsVerificationError(#[from] ClaimsVerificationError),
    #[error("Server did not return an ID token")]
    OpenidIdTokenError,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct OidcAuthCfg {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: Option<String>,
    pub scopes: Option<String>,
    pub username_claim: Option<String>,
    pub groupinfo_claim: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OidcClient {
    client: CoreClient,
    pub authorize_url: String,
    nonce: Nonce,
    username_claim: Option<String>,
    groupinfo_claim: String,
}

impl OidcClient {
    pub async fn from_config(cfg: &OidcAuthCfg) -> Self {
        info!(
            "Fetching {}/.well-known/openid-configuration",
            &cfg.issuer_url
        );
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(cfg.issuer_url.clone()).expect("Invalid issuer URL"),
            async_http_client,
        )
        .await
        .expect("Failed to discover OpenID Provider");

        // Set up the config for the OAuth2 process.
        let redirect_uri = cfg
            .redirect_uri
            .clone()
            .unwrap_or("http://127.0.0.1:8080/auth".to_string());
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(cfg.client_id.clone()),
            Some(ClientSecret::new(cfg.client_secret.clone())),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri).expect("Invalid redirect URL"));

        // Generate the authorization URL to which we'll redirect the user.
        let mut auth_client = client.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );
        let scopes = cfg.scopes.clone().unwrap_or("email profile".to_string());
        for scope in scopes.split(' ') {
            auth_client = auth_client.add_scope(Scope::new(scope.to_string()));
        }
        let (authorize_url, _csrf_state, nonce) = auth_client.url();
        let groupinfo_claim = cfg.groupinfo_claim.clone().unwrap_or("group".to_string());
        OidcClient {
            client,
            authorize_url: authorize_url.to_string(),
            nonce,
            username_claim: cfg.username_claim.clone(),
            groupinfo_claim,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AuthRequest {
    pub code: String,
    // pub state: String,
    // pub scope: String,
}

impl AuthRequest {
    pub async fn auth(&self, oidc: &OidcClient) -> Result<Identity, AuthError> {
        // let state = CsrfToken::new(self.state.clone());
        let code = AuthorizationCode::new(self.code.clone());
        // Exchange the code with a token.
        let token_response = oidc
            .client
            .exchange_code(code)
            .request_async(async_http_client)
            .await?;
        debug!("IdP returned scopes: {:?}", token_response.scopes());

        let id_token_verifier = oidc.client.id_token_verifier();
        let id_token_claims = token_response
            .extra_fields()
            .id_token()
            .ok_or(AuthError::OpenidIdTokenError)?
            .claims(&id_token_verifier, &oidc.nonce)?;

        // Convert back to raw JSON to simplify extracting configurable claims
        let userinfo = serde_json::to_value(id_token_claims).unwrap();
        info!("userinfo: {userinfo:#?}");

        let username = if let Some(claim) = &oidc.username_claim {
            userinfo[claim].as_str()
        } else {
            userinfo
                .get("preferred_username")
                .or(userinfo.get("upn"))
                .or(userinfo.get("email"))
                .and_then(|v| v.as_str())
        }
        .unwrap_or("")
        .to_string();
        let groups = match &userinfo[&oidc.groupinfo_claim] {
            Value::String(s) => vec![s.as_str().to_string()],
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            _ => Vec::new(),
        };
        Ok(Identity { username, groups })
    }
}
