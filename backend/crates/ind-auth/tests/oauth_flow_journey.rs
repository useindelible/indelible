#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use chrono::Utc;
use ind_application::repos::refresh_token::RefreshTokenRepository;
use ind_auth::{
    AuthError, AuthorizationCodeService, NativeOAuthFlow, RefreshTokenService, StoredOAuthFlow,
    StoredOAuthFlowKind, consume_oauth_flow, hash_token, open_oauth_flow, seal_oauth_flow,
    store_oauth_flow,
};
use ind_domain::{ClientType, RefreshToken, RefreshTokenId};
use ind_persistence::repos::{
    PgAuthorizationCodeRepository, PgOAuthFlowRepository, PgRefreshTokenRepository,
};
use ind_test_support::{TestDb, UserFactory};

fn native_flow(state: &str, expires_at: i64) -> StoredOAuthFlow {
    StoredOAuthFlow {
        provider: "oidc".into(),
        csrf_state: state.into(),
        issuer: Some("https://identity.example.com".into()),
        oidc_flow: None,
        kind: StoredOAuthFlowKind::Native(NativeOAuthFlow {
            platform: ClientType::Desktop,
            redirect_uri: "indelible://oauth/callback".into(),
            code_challenge: "s256-challenge".into(),
            app_state: "return-to-library".into(),
        }),
        expires_at,
    }
}

#[tokio::test]
async fn authorization_codes_persist_without_lifecycle_scopes() {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use sha2::{Digest, Sha256};

    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let code_repo = Arc::new(PgAuthorizationCodeRepository::new(db.pool().clone()));
    let refresh_repo = Arc::new(PgRefreshTokenRepository::new(db.pool().clone()));
    let refresh_service = Arc::new(RefreshTokenService::new(
        refresh_repo,
        b"test-jwt-secret-at-least-32-bytes".to_vec(),
    ));
    let redirect_uri = "indelible://oauth/callback";
    let service =
        AuthorizationCodeService::new(code_repo, refresh_service, vec![redirect_uri.to_string()]);
    let verifier = "authorization-code-verifier";
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    let issued = service
        .create_code(
            user.id,
            ClientType::Desktop,
            challenge,
            "S256".to_string(),
            redirect_uri.to_string(),
        )
        .await
        .unwrap();

    let scopes_column_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
             SELECT 1 FROM information_schema.columns \
             WHERE table_schema = 'public' \
               AND table_name = 'authorization_codes' \
               AND column_name = 'scopes'\
         )",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(!scopes_column_exists);

    let exchanged = service
        .exchange_code(
            &issued.raw_code,
            verifier,
            redirect_uri,
            ClientType::Desktop,
            None,
            None,
        )
        .await
        .unwrap();
    assert!(exchanged.access_token.starts_with("eyJ"));
    assert!(!exchanged.refresh_token.is_empty());
}

#[tokio::test]
async fn oauth_state_is_sealed_expiring_and_consumed_exactly_once() {
    let db = TestDb::new().await;
    let repo = Arc::new(PgOAuthFlowRepository::new(db.pool().clone()));
    let secret = b"test-session-secret-with-enough-entropy";
    let state = "opaque-state-value";
    let flow = native_flow(
        state,
        (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
    );

    let sealed = seal_oauth_flow(&flow, secret).unwrap();
    assert!(
        !sealed
            .windows(state.len())
            .any(|bytes| bytes == state.as_bytes())
    );
    let opened = open_oauth_flow(&sealed, secret).unwrap();
    assert_eq!(opened.csrf_state, state);
    assert!(matches!(opened.kind, StoredOAuthFlowKind::Native(_)));

    let mut tampered = sealed;
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert!(open_oauth_flow(&tampered, secret).is_err());
    assert!(open_oauth_flow(&[2, 0, 1], secret).is_err());

    store_oauth_flow(repo.as_ref(), &flow, secret)
        .await
        .unwrap();
    let (first, second) = tokio::join!(
        consume_oauth_flow(repo.as_ref(), state, secret),
        consume_oauth_flow(repo.as_ref(), state, secret),
    );
    let consumed = [first.unwrap(), second.unwrap()];
    assert_eq!(consumed.iter().filter(|flow| flow.is_some()).count(), 1);
    let consumed = consumed.into_iter().flatten().next().unwrap();
    assert_eq!(consumed.provider, "oidc");
    assert_eq!(
        consumed.issuer.as_deref(),
        Some("https://identity.example.com")
    );

    let expired_state = "expired-state";
    let expired = native_flow(
        expired_state,
        (Utc::now() - chrono::Duration::seconds(1)).timestamp(),
    );
    store_oauth_flow(repo.as_ref(), &expired, secret)
        .await
        .unwrap();
    assert!(
        consume_oauth_flow(repo.as_ref(), expired_state, secret)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn refresh_tokens_rotate_once_revoke_by_family_and_honor_both_expiry_clocks() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let repo = Arc::new(PgRefreshTokenRepository::new(db.pool().clone()));
    let service =
        RefreshTokenService::new(repo.clone(), b"test-jwt-secret-at-least-32-bytes".to_vec());

    let issued = service
        .issue_tokens(user.id, ClientType::Cli, Some("127.0.0.1".into()), None)
        .await
        .unwrap();
    let rotated = service
        .rotate(&issued.raw_refresh_token, None, Some("ind-cli".into()))
        .await
        .unwrap();
    assert_ne!(rotated.raw_refresh_token, issued.raw_refresh_token);
    assert!(rotated.access_token.starts_with("eyJ"));
    let grace = service
        .rotate(&issued.raw_refresh_token, None, None)
        .await
        .unwrap();
    assert!(grace.raw_refresh_token.is_empty());

    service
        .revoke_family_by_token(&rotated.raw_refresh_token)
        .await
        .unwrap();
    assert!(matches!(
        service.rotate(&rotated.raw_refresh_token, None, None).await,
        Err(AuthError::TokenRevoked)
    ));

    for expire_absolute in [false, true] {
        let raw = format!("indr_expired-{expire_absolute}");
        let now = Utc::now();
        repo.create(RefreshToken {
            id: RefreshTokenId::new(),
            family_id: uuid::Uuid::now_v7(),
            user_id: user.id,
            token_hash: hash_token(&raw),
            client_type: ClientType::Web,
            ip_address: None,
            user_agent: None,
            replaced_by: None,
            revoked_at: None,
            expires_at: if expire_absolute {
                now + chrono::Duration::days(1)
            } else {
                now - chrono::Duration::seconds(1)
            },
            absolute_expires_at: if expire_absolute {
                now - chrono::Duration::seconds(1)
            } else {
                now + chrono::Duration::days(1)
            },
            last_used_at: now,
            created_at: now,
        })
        .await
        .unwrap();
        assert!(matches!(
            service.rotate(&raw, None, None).await,
            Err(AuthError::TokenExpired)
        ));
    }
}
