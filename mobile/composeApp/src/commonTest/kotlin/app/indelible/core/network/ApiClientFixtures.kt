package app.indelible.core.network

internal fun sessionProfileJson() =
    """
    {
        "id": "usr_01ABCDEF",
        "object": "user",
        "email": "user@example.com",
        "display_name": "Test",
        "email_verified": true,
        "onboarding_completed": true,
        "has_password": true,
        "locale": "en",
        "theme": "auto",
        "timezone": "UTC",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }
    """.trimIndent()
