package app.indelible.i18ncheck

import java.nio.file.Files
import kotlin.io.path.writeText
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class MobileI18nCheckerTest {
    @Test
    fun kotlinCheckerFindsDirectAndLocallyAliasedVisibleCopy() {
        val source =
            """
            @Composable
            fun Screen() {
                Text("Direct copy")
                val copy = "Aliased copy"
                Text(copy)
                Text(stringResource(Res.string.common_done))
            }

            @Preview
            @Composable
            fun PreviewScreen() {
                Text("Preview copy")
            }
            """.trimIndent()

        val errors = KotlinSourceChecker().use { it.check("Screen.kt", source) }

        assertEquals(2, errors.size)
        assertTrue(errors.any { "Direct copy" in it })
        assertTrue(errors.any { "Aliased copy" in it })
    }

    @Test
    fun kotlinCheckerCoversEmbeddedHtmlPluralAndResourceCasing() {
        val source =
            listOf(
                "fun markup(count: Int) = \"\"\"<button>Read more</button>\"\"\"",
                "val countLabel = \"${'$'}count item${'$'}{if (count == 1) \\\"\\\" else \\\"s\\\"}\"",
                "val heading = stringResource(Res.string.common_done).uppercase()",
            ).joinToString("\n")

        val errors = KotlinSourceChecker().use { it.check("ReaderHtmlMarkup.kt", source) }

        assertTrue(errors.any { "raw visible embedded HTML" in it })
        assertTrue(errors.any { "manual English plural suffix" in it })
        assertTrue(errors.any { "case transformation applied to localized text" in it })
    }

    @Test
    fun swiftCheckerAcceptsCatalogKeysAndRejectsRawCopy() {
        val source =
            """
            Text("share_save")
            // Button("Commented out") {}
            /* Text("Also commented out") */
            Button("Cancel") {}
            """.trimIndent()

        val errors = SwiftSourceChecker(setOf("share_save")).check("ShareView.swift", source)

        assertEquals(1, errors.size)
        assertContains(errors.single(), "Cancel")
    }

    @Test
    fun xmlCatalogReaderRejectsDoctypeAndExternalEntities() {
        val secret = Files.createTempFile("i18n-secret", ".txt").apply { writeText("not-for-ci") }
        val catalog =
            Files.createTempFile("strings", ".xml").apply {
                writeText(
                    """
                    <?xml version="1.0"?>
                    <!DOCTYPE resources [<!ENTITY leak SYSTEM="${secret.toUri()}">]>
                    <resources><string name="common_done">&leak;</string></resources>
                    """.trimIndent(),
                )
            }

        val error = runCatching { XmlCatalogReader(ALLOWED_PREFIXES).read(catalog.toFile()) }.exceptionOrNull()

        assertTrue(error != null)
        assertTrue(error.message.orEmpty().contains("DOCTYPE", ignoreCase = true))
    }

    private companion object {
        val ALLOWED_PREFIXES = setOf("common_")
    }
}
