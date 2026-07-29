package app.indelible.navigation

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CalendarMonth
import androidx.compose.material.icons.filled.CollectionsBookmark
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.RssFeed
import androidx.compose.material.icons.filled.Search
import androidx.compose.ui.graphics.vector.ImageVector

enum class TabItem(
    val route: String,
    val label: String,
    val icon: ImageVector,
) {
    HOME("home", "Home", Icons.Filled.Home),
    LIBRARY("library", "Library", Icons.Filled.CollectionsBookmark),
    FEED("feed", "Feed", Icons.Filled.RssFeed),
    SEARCH("search", "Search", Icons.Filled.Search),
    REVIEW("review", "Review", Icons.Filled.CalendarMonth),
    PROFILE("profile", "Profile", Icons.Filled.Person),
}
