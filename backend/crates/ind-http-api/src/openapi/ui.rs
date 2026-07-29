use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::tag::Tag;
use utoipa::{Modify, OpenApi};

use super::ApiDoc;

pub(super) struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "session_cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("refresh_token"))),
        );
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "api_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("ind_*")
                    .build(),
            ),
        );
        openapi.tags = Some(tags());
    }
}

fn tags() -> Vec<Tag> {
    vec![
        tag("Auth", "Authentication and session management"),
        tag("Account", "Profile and account management"),
        tag("Settings", "User settings pages and preferences"),
        tag("Onboarding", "User onboarding flow"),
        tag("API Tokens", "API token management"),
        tag("Extension", "Browser extension integration"),
        tag(
            "Items",
            "Item CRUD, triage, favorites, shortlist, and trash",
        ),
        tag("Events", "Realtime domain event stream"),
        tag(
            "Archive Assets",
            "Archive asset management and download URLs",
        ),
        tag(
            "Asset Proxy",
            "Stream archive assets through the API server",
        ),
        tag("Feeds", "Feed subscriptions and reader"),
        tag("Highlights", "Highlight CRUD, notes, and export"),
        tag("Home", "Home dashboard aggregation and widget settings"),
        tag(
            "Mila",
            "Mila configuration, status, AI outputs, and manual actions",
        ),
        tag(
            "Entities",
            "Entity summaries, detail views, item browsing, and merge/rename",
        ),
        tag("Collections", "Collection CRUD and item membership"),
        tag("Tags", "Tag CRUD, rename, recolor, and merge"),
        tag("Smart Lists", "Smart list CRUD and filter evaluation"),
        tag(
            "TTS",
            "Text-to-speech voice personas, sessions, and audio streaming",
        ),
        tag(
            "Integrations",
            "Integration connections and OAuth provider linking",
        ),
        tag("Imports", "Item import jobs: upload, progress, rollback"),
        tag("Export", "Scoped export endpoints for external plugins"),
    ]
}

fn tag(name: &str, description: &str) -> Tag {
    let mut tag = Tag::new(name);
    tag.description = Some(description.to_string());
    tag
}

pub fn swagger_ui() -> utoipa_swagger_ui::SwaggerUi {
    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

pub fn scalar_ui() -> utoipa_scalar::Scalar<utoipa::openapi::OpenApi> {
    use utoipa_scalar::Servable;
    utoipa_scalar::Scalar::with_url("/scalar", ApiDoc::openapi())
}
