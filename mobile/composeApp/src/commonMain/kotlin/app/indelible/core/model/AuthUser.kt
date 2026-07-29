package app.indelible.core.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class AuthUser(
    val id: String,
    val email: String,
    @SerialName("display_name")
    val displayName: String,
    @SerialName("email_verified")
    val emailVerified: Boolean = false,
    @SerialName("onboarding_completed")
    val onboardingCompleted: Boolean = false,
    @SerialName("ingest_email")
    val ingestEmail: String? = null,
    @SerialName("ingest_library_email")
    val ingestLibraryEmail: String? = null,
    @SerialName("avatar_url")
    val avatarUrl: String? = null,
)
