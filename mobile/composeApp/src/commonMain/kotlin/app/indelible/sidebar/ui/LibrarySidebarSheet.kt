package app.indelible.sidebar.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.collections.ui.collectionBannerIndex
import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList
import app.indelible.sidebar.ui.components.SidebarAddPlaceholder
import app.indelible.sidebar.ui.components.SidebarFooter
import app.indelible.sidebar.ui.components.SidebarGroupLabel
import app.indelible.sidebar.ui.components.SidebarNavItem
import app.indelible.sidebar.ui.components.SidebarProfileHeader
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.datetime.Instant
import kotlinx.serialization.json.JsonObject

/**
 * The navigation drawer (prototype `mobile-sidebar-reimagined.html`): a fixed
 * profile header, a scrollable body of grouped destinations (Library content
 * types, then Collections and Smart Lists driven by [collections]/[smartLists]),
 * and a fixed Settings/Trash footer. Empty Collection/Smart-List groups render a
 * ghost "New …" placeholder instead of rows.
 */
@Composable
fun LibrarySidebarSheet(
    displayName: String,
    currentRoute: String?,
    currentContentType: String?,
    collections: List<Collection>,
    smartLists: List<SmartList>,
    onNavigateToContentType: (String?) -> Unit,
    onNavigateToCollection: (String) -> Unit,
    onNavigateToSmartList: (String) -> Unit,
    onNewCollection: () -> Unit,
    onNewSmartList: () -> Unit,
    onNavigateToSettings: () -> Unit,
    onNavigateToTrash: () -> Unit,
    modifier: Modifier = Modifier,
    subtitle: String = "",
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    val tagColors = IndelibleTheme.colors.tagColors

    Column(modifier = modifier.fillMaxHeight()) {
        SidebarProfileHeader(
            displayName = displayName,
            subtitle = subtitle,
            avatarUrl = avatarUrl,
            avatarBytes = avatarBytes,
        )

        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(
                        horizontal = IndelibleSpacing.step12,
                        vertical = IndelibleSpacing.step8,
                    ),
        ) {
            SidebarGroupLabel("Library")
            SidebarNavItem(
                label = "All items",
                active = currentContentType == null && currentRoute == "library",
                onClick = { onNavigateToContentType(null) },
                icon = IndelibleIcons.Grid,
            )
            ContentTypeEntries.forEach { (label, apiValue, icon) ->
                SidebarNavItem(
                    label = label,
                    active = currentContentType == apiValue,
                    onClick = { onNavigateToContentType(apiValue) },
                    icon = icon,
                )
            }

            SidebarGroupLabel("Collections")
            if (collections.isEmpty()) {
                SidebarAddPlaceholder(label = "New collection", onClick = onNewCollection)
            } else {
                collections.forEach { collection ->
                    SidebarNavItem(
                        label = collection.name,
                        active = false,
                        onClick = { onNavigateToCollection(collection.id) },
                        dotColor = tagColors[collectionBannerIndex(collection.color, collection.id)],
                        count = collection.itemCount.toInt(),
                    )
                }
            }

            SidebarGroupLabel("Smart Lists")
            if (smartLists.isEmpty()) {
                SidebarAddPlaceholder(label = "New smart list", onClick = onNewSmartList)
            } else {
                smartLists.forEach { smartList ->
                    SidebarNavItem(
                        label = smartList.name,
                        active = false,
                        onClick = { onNavigateToSmartList(smartList.id) },
                        icon = IndelibleIcons.SmartList,
                        iconTint = tagColors[collectionBannerIndex(smartList.color, smartList.id)],
                    )
                }
            }
        }

        SidebarFooter(onSettings = onNavigateToSettings, onTrash = onNavigateToTrash)
    }
}

// (label, content-type API value, leading icon) for the Library group rows.
private val ContentTypeEntries =
    listOf(
        Triple("Articles", "article", IndelibleIcons.Article),
        Triple("Books", "book", IndelibleIcons.Book),
        Triple("Emails", "email", IndelibleIcons.Email),
        Triple("PDFs", "pdf", IndelibleIcons.Pdf),
        Triple("Tweets", "tweet", IndelibleIcons.Tweet),
        Triple("Videos", "video", IndelibleIcons.Video),
    )

private fun sampleCollections() =
    listOf(
        Collection(
            color = "blue",
            createdAt = Instant.fromEpochMilliseconds(0),
            id = "col_1",
            itemCount = 14,
            name = "Reading list",
            `object` = "collection",
            sortOrder = 0,
            updatedAt = Instant.fromEpochMilliseconds(0),
        ),
        Collection(
            color = "purple",
            createdAt = Instant.fromEpochMilliseconds(0),
            id = "col_2",
            itemCount = 26,
            name = "Work",
            `object` = "collection",
            sortOrder = 1,
            updatedAt = Instant.fromEpochMilliseconds(0),
        ),
    )

private fun sampleSmartLists() =
    listOf(
        SmartList(
            color = "amber",
            createdAt = Instant.fromEpochMilliseconds(0),
            filterExpression = JsonObject(emptyMap()),
            id = "sl_1",
            isPinned = false,
            name = "Unread",
            `object` = "smart_list",
            updatedAt = Instant.fromEpochMilliseconds(0),
        ),
    )

@androidx.compose.ui.tooling.preview.Preview
@Composable
private fun LibrarySidebarSheetPopulatedPreview() {
    AppTheme(darkTheme = false) {
        Surface(color = MaterialTheme.colorScheme.surface) {
            LibrarySidebarSheet(
                displayName = "Samuel Ajisegiri",
                currentRoute = "library",
                currentContentType = null,
                collections = sampleCollections(),
                smartLists = sampleSmartLists(),
                onNavigateToContentType = {},
                onNavigateToCollection = {},
                onNavigateToSmartList = {},
                onNewCollection = {},
                onNewSmartList = {},
                onNavigateToSettings = {},
                onNavigateToTrash = {},
                subtitle = "289 saved items",
            )
        }
    }
}

@androidx.compose.ui.tooling.preview.Preview
@Composable
private fun LibrarySidebarSheetEmptyPreview() {
    AppTheme(darkTheme = true) {
        Surface(color = MaterialTheme.colorScheme.surface) {
            LibrarySidebarSheet(
                displayName = "Samuel Ajisegiri",
                currentRoute = "library",
                currentContentType = "article",
                collections = emptyList(),
                smartLists = emptyList(),
                onNavigateToContentType = {},
                onNavigateToCollection = {},
                onNavigateToSmartList = {},
                onNewCollection = {},
                onNewSmartList = {},
                onNavigateToSettings = {},
                onNavigateToTrash = {},
                subtitle = "289 saved items",
            )
        }
    }
}
