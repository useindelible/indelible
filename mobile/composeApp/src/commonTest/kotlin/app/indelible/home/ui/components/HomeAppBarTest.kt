package app.indelible.home.ui.components

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class HomeAppBarTest {
    @Test
    fun theBarCoversTheContentScrollingUnderIt() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    HomeAppBar(
                        userDisplayName = "Sam",
                        onMenuClick = {},
                        onSearchClick = {},
                        onProfileClick = {},
                        containerColor = Color.Red,
                    )
                }
            }

            val pixels = onRoot().captureToImage().toPixelMap()

            assertEquals(Color.Red, pixels[pixels.width / 2, 1])
        }
}
