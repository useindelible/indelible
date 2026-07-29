use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationNotionOAuthSettings {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IntegrationOAuthSettings {
    pub notion: Option<IntegrationNotionOAuthSettings>,
}
