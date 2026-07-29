package app.indelible.share.model

import kotlinx.serialization.Serializable

@Serializable
data class PendingItem(
    val id: String,
    val url: String,
    val enqueuedAt: Long,
)
