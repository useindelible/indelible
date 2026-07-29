#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_egress::{EgressError, EgressPolicy, UrlRules, validate_url};

fn opted_in() -> EgressPolicy {
    EgressPolicy {
        allow_private_targets: true,
        extra_allowed_ips: Vec::new(),
    }
}

/// Docker's host-gateway names are as unambiguously private as `localhost`:
/// an operator who opted into private targets reaches on-host services (e.g.
/// a local AI server) from inside the shipped containers.
#[test]
fn docker_internal_http_is_allowed_when_private_targets_are_opted_in() {
    let url = validate_url(
        "http://host.docker.internal:11434",
        &UrlRules::ai_endpoint(),
        &opted_in(),
    )
    .unwrap();
    assert_eq!(url.host_str(), Some("host.docker.internal"));
}

#[test]
fn docker_internal_stays_blocked_without_the_opt_in() {
    let err = validate_url(
        "http://host.docker.internal:11434",
        &UrlRules::ai_endpoint(),
        &EgressPolicy::strict(),
    )
    .unwrap_err();
    assert!(
        matches!(err, EgressError::HostNotAllowed { .. }),
        "expected the private-target gate, got {err:?}"
    );
}

/// The BYOK cleartext protection is deliberate and survives the opt-in: a
/// public host must always speak https, no matter what the policy allows.
#[test]
fn public_hosts_still_require_https_even_with_private_targets_allowed() {
    let err = validate_url(
        "http://example.com/v1",
        &UrlRules::ai_endpoint(),
        &opted_in(),
    )
    .unwrap_err();
    assert!(
        matches!(err, EgressError::SchemeRequiresHttps),
        "expected the https requirement, got {err:?}"
    );
}

#[test]
fn localhost_http_keeps_its_existing_opt_in_behavior() {
    assert!(
        validate_url(
            "http://localhost:11434",
            &UrlRules::ai_endpoint(),
            &opted_in()
        )
        .is_ok()
    );
    assert!(matches!(
        validate_url(
            "http://localhost:11434",
            &UrlRules::ai_endpoint(),
            &EgressPolicy::strict()
        ),
        Err(EgressError::HostNotAllowed { .. })
    ));
}
