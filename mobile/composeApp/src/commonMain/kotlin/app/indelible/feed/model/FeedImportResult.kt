package app.indelible.feed.model

import app.indelible.api.generated.models.OpmlImportResponse

data class OpmlImportResult(
    val created: Int,
    val errors: List<String>,
    val skipped: Int,
)

fun OpmlImportResponse.toOpmlImportResult(): OpmlImportResult =
    OpmlImportResult(
        created = created,
        errors = errors,
        skipped = skipped,
    )
