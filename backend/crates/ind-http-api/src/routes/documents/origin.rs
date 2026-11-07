use ind_domain::{ClientId, EventOrigin};

use crate::middleware::{ApiCredential, Principal};

/// A supplied device id wins; otherwise the event is attributed to the caller's credential.
pub(crate) fn origin_from(principal: &Principal, device: Option<ClientId>) -> EventOrigin {
    if let Some(client_id) = device {
        return EventOrigin::Device(client_id);
    }
    match &principal.credential {
        ApiCredential::UserAccessJwt { client_type } => EventOrigin::Surface(*client_type),
        ApiCredential::PersonalAccessToken { token_id, .. } => EventOrigin::ApiToken(*token_id),
    }
}
