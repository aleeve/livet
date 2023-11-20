use leptos::*;
use leptos_oidc::{Auth, AuthParameters};

#[component()]
pub fn Auth(children: Children) -> impl IntoView {
    let auth_parameters = AuthParameters {
        auth_endpoint: std::option_env!("AUTH_ENDPOINT")
            .unwrap_or("https://dev-qcuxgjrapycf5ib4.us.auth0.com/authorize")
            .to_string(),
        token_endpoint: std::option_env!("AUTH_TOKEN_ENDPOINT")
            .unwrap_or("https://dev-qcuxgjrapycf5ib4.us.auth0.com/oauth/token")
            .to_string(),
        logout_endpoint: "https://dev-qcuxgjrapycf5ib4.us.auth0.com/oidc/logout".to_string(),
        client_id: std::option_env!("AUTH_CLIENT_ID")
            .unwrap_or("DGDqdTkVhH465k1aniWOoExz22skprju")
            .to_string(),
        redirect_uri: std::option_env!("AUTH_REDIRECT_URL")
            .unwrap_or("http://localhost:8080")
            .to_string(),
        post_logout_redirect_uri: std::option_env!("AUTH_LOGOUT_REDIRECT")
            .unwrap_or("http://localhost:8080")
            .to_string(),
        scope: Some("openid".to_string()),
    };

    let _auth = Auth::init(auth_parameters);

    view! {
        {children()}
    }
}
