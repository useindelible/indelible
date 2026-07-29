#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::IpAddr;

use ind_egress::{EgressPolicy, GuardedClientOptions, UrlRules, build_guarded_client};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn loopback() -> IpAddr {
    "127.0.0.1".parse().unwrap()
}

#[tokio::test]
async fn guarded_client_enforces_strict_pinned_and_redirect_boundaries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ok"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/start"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", "http://10.0.0.1/internal"),
        )
        .mount(&server)
        .await;

    let strict = build_guarded_client(GuardedClientOptions::new(
        UrlRules::ingest(),
        EgressPolicy::strict(),
    ))
    .expect("strict client builds");
    assert!(strict.get(&server.uri()).is_err());
    assert!(
        strict
            .get("http://169.254.169.254/latest/meta-data/")
            .is_err()
    );
    assert!(server.received_requests().await.unwrap().is_empty());

    let policy = EgressPolicy {
        allow_private_targets: false,
        extra_allowed_ips: vec![loopback()],
    };
    let pinned = build_guarded_client(GuardedClientOptions::new(UrlRules::ingest(), policy))
        .expect("pinned client builds");
    pinned
        .get(&format!("{}/ok", server.uri()))
        .unwrap()
        .send()
        .await
        .unwrap();
    assert!(
        pinned
            .get(&format!("{}/start", server.uri()))
            .unwrap()
            .send()
            .await
            .is_err()
    );
}
