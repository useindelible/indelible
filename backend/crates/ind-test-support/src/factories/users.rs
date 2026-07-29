use super::prelude::*;

#[derive(Default)]
pub struct UserFactory {
    email_verified: bool,
}

impl UserFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_email_verified(mut self, verified: bool) -> Self {
        self.email_verified = verified;
        self
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> User {
        let timestamp = Utc::now();
        PgUserRepository::new(pool.clone())
            .create(User {
                id: UserId::new(),
                email: format!("test-{}@example.com", short_unique_suffix()),
                password_hash: None,
                display_name: format!("{} {}", Name().fake::<String>(), short_unique_suffix()),
                avatar_url: None,
                locale: "en".into(),
                timezone: "UTC".into(),
                theme: Theme::System,
                email_verified: self.email_verified,
                onboarding_completed: false,
                onboarding_step: 0,
                email_token: ind_auth::crypto::generate_email_token(),
                status: UserStatus::Active,
                created_at: timestamp,
                updated_at: timestamp,
            })
            .await
            .expect("UserFactory::insert failed")
    }
}
