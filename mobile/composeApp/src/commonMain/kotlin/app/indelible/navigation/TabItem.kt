package app.indelible.navigation

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CalendarMonth
import androidx.compose.material.icons.filled.CollectionsBookmark
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.RssFeed
import androidx.compose.material.icons.filled.Search
import androidx.compose.ui.graphics.vector.ImageVector
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.nav_feed
import indelible.composeapp.generated.resources.nav_home
import indelible.composeapp.generated.resources.nav_library
import indelible.composeapp.generated.resources.nav_profile
import indelible.composeapp.generated.resources.nav_review
import indelible.composeapp.generated.resources.nav_search
import org.jetbrains.compose.resources.StringResource

enum class TabItem(
    val route: String,
    val labelRes: StringResource,
    val icon: ImageVector,
) {
    HOME("home", Res.string.nav_home, Icons.Filled.Home),
    LIBRARY("library", Res.string.nav_library, Icons.Filled.CollectionsBookmark),
    FEED("feed", Res.string.nav_feed, Icons.Filled.RssFeed),
    SEARCH("search", Res.string.nav_search, Icons.Filled.Search),
    REVIEW("review", Res.string.nav_review, Icons.Filled.CalendarMonth),
    PROFILE("profile", Res.string.nav_profile, Icons.Filled.Person),
}
