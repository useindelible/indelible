package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.DocumentEntity
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.geistMonoFontFamily
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_entities_organizations
import indelible.composeapp.generated.resources.reader_entities_other
import indelible.composeapp.generated.resources.reader_entities_people
import indelible.composeapp.generated.resources.reader_entities_topics
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/**
 * Mila-extracted entities for the item record, grouped by type so people read
 * apart from organizations. Entity types beyond the known three fall into a
 * trailing "Other" group rather than disappearing.
 */
@Composable
internal fun EntityGroups(entities: List<DocumentEntity>) {
    val byType = entities.groupBy { it.entityType.lowercase() }
    val known = ENTITY_GROUPS.map { it.first }.toSet()
    val other = entities.filter { it.entityType.lowercase() !in known }
    val groups =
        ENTITY_GROUPS.mapNotNull { (type, labelRes) -> byType[type]?.let { labelRes to it } } +
            if (other.isEmpty()) emptyList() else listOf(Res.string.reader_entities_other to other)

    Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14)) {
        groups.forEach { (labelRes, group) -> EntityGroup(labelRes = labelRes, entities = group) }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun EntityGroup(
    labelRes: StringResource,
    entities: List<DocumentEntity>,
) {
    Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
        Text(
            text = stringResource(labelRes),
            style = monoLabelStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            entities.forEach { entity -> EntityChip(entity) }
        }
    }
}

/** The trailing count is how many other documents mention the entity, so it is only shown above one. */
@Composable
private fun EntityChip(entity: DocumentEntity) {
    Row(
        modifier =
            Modifier
                .clip(IndelibleShape.sm)
                .background(MaterialTheme.colorScheme.secondaryContainer)
                .padding(
                    horizontal = IndelibleSpacing.step10,
                    vertical = IndelibleSpacing.step6,
                ),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = entity.name,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSecondaryContainer,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (entity.itemCount > 1) {
            Text(
                text = entity.itemCount.toString(),
                style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
                color = IndelibleTheme.colors.textTertiary,
            )
        }
    }
}

/** Group order mirrors the reader prototype: people, then organizations, then topics. */
private val ENTITY_GROUPS =
    listOf(
        "person" to Res.string.reader_entities_people,
        "organization" to Res.string.reader_entities_organizations,
        "topic" to Res.string.reader_entities_topics,
    )

// ============================================================
// Previews
// ============================================================

@Suppress("MagicNumber") // preview-only sample counts
internal val previewEntities =
    listOf(
        previewEntity("ent_1", "Sir Ken Robinson", "person", 3),
        previewEntity("ent_2", "Gillian Lynne", "person", 1),
        previewEntity("ent_3", "TED", "organization", 9),
        previewEntity("ent_4", "Creativity", "topic", 5),
        previewEntity("ent_5", "Education reform", "topic", 7),
    )

private fun previewEntity(
    id: String,
    name: String,
    type: String,
    itemCount: Long,
): DocumentEntity =
    DocumentEntity(
        createdAt = Instant.DISTANT_PAST,
        entityType = type,
        firstSeenAt = Instant.DISTANT_PAST,
        id = id,
        itemCount = itemCount,
        lastSeenAt = Instant.DISTANT_PAST,
        name = name,
        `object` = "entity",
        totalMentions = itemCount,
    )

@Preview
@Composable
private fun EntityGroupsPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            EntityGroups(previewEntities)
        }
    }
}

@Preview
@Composable
private fun EntityGroupsPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            EntityGroups(previewEntities)
        }
    }
}
