package app.indelible.i18ncheck

import java.io.File
import kotlin.system.exitProcess

fun main(args: Array<String>) {
    require(args.size == 2) { "Expected compose-app and iOS share-extension directories" }
    val report = MobileI18nChecker(File(args[0]), File(args[1])).check()
    if (report.errors.isNotEmpty()) {
        System.err.println(report.errors.joinToString(prefix = "Mobile i18n check failed:\n- ", separator = "\n- "))
        exitProcess(1)
    }
    println(
        "Mobile i18n check passed " +
            "(${report.composeResources} Compose resources, ${report.translations} translations, " +
            "${report.shareResources} iOS share resources)",
    )
}

data class MobileI18nReport(
    val composeResources: Int,
    val translations: Int,
    val shareResources: Int,
    val errors: List<String>,
)

class MobileI18nChecker(
    private val composeAppDir: File,
    private val shareExtensionDir: File,
) {
    fun check(): MobileI18nReport {
        val errors = mutableListOf<String>()
        val resourcesRoot = composeAppDir.resolve("src/commonMain/composeResources")
        val source = readCatalog(resourcesRoot.resolve("values/strings.xml"), errors)
        val translations =
            resourcesRoot
                .listFiles()
                .orEmpty()
                .filter { it.isDirectory && it.name.startsWith("values-") }
                .map { it.resolve("strings.xml") }
                .filter(File::isFile)
                .sortedBy(File::getPath)

        translations.forEach { file -> checkTranslation(source, file, errors) }

        val shareEnglish = readStringsCatalog(shareExtensionDir.resolve("en.lproj/Localizable.strings"), errors)
        val shareFrench = readStringsCatalog(shareExtensionDir.resolve("fr.lproj/Localizable.strings"), errors)
        checkShareCatalogs(shareEnglish, shareFrench, errors)
        checkSources(shareEnglish.keys, errors)

        return MobileI18nReport(source.size, translations.size, shareEnglish.size, errors)
    }

    private fun readCatalog(
        file: File,
        errors: MutableList<String>,
    ): Map<String, CatalogResource> =
        runCatching { XmlCatalogReader(ALLOWED_PREFIXES).read(file) }
            .fold(
                onSuccess = {
                    errors += it.errors
                    it.resources
                },
                onFailure = {
                    errors += "${file.path}: XML catalog rejected: ${it.message.orEmpty()}"
                    emptyMap()
                },
            )

    private fun checkTranslation(
        source: Map<String, CatalogResource>,
        file: File,
        errors: MutableList<String>,
    ) {
        val translation = readCatalog(file, errors)
        val locale = file.parentFile.name.removePrefix("values-")
        val extra = translation.keys - source.keys
        val missing = source.keys - translation.keys
        if (extra.isNotEmpty()) errors += "$locale: unknown resources ${extra.sorted().joinToString()}"
        if (missing.isNotEmpty()) errors += "$locale: missing resources ${missing.sorted().joinToString()}"

        (source.keys intersect translation.keys).forEach { key ->
            val sourceResource = source.getValue(key)
            val translatedResource = translation.getValue(key)
            if (sourceResource.type != translatedResource.type) errors += "$locale: resource type differs for $key"
            if (sourceResource.values.keys != translatedResource.values.keys) {
                errors += "$locale: plural quantities differ for $key"
            }
            (sourceResource.values.keys intersect translatedResource.values.keys).forEach { quantity ->
                if (placeholders(sourceResource.values.getValue(quantity)) !=
                    placeholders(translatedResource.values.getValue(quantity))
                ) {
                    errors += "$locale: placeholders differ for $key ($quantity)"
                }
            }
        }
    }

    private fun readStringsCatalog(
        file: File,
        errors: MutableList<String>,
    ): Map<String, String> {
        val entryPattern = Regex("""^\s*"([^"]+)"\s*=\s*"((?:\\.|[^"])*)";\s*$""")
        val catalog = linkedMapOf<String, String>()
        file.readLines().forEachIndexed { index, line ->
            val trimmed = line.trim()
            if (trimmed.isEmpty() || trimmed.startsWith("//") || trimmed.startsWith("/*")) return@forEachIndexed
            val match = entryPattern.matchEntire(line)
            if (match == null) {
                errors += "${file.path}:${index + 1}: invalid Localizable.strings entry"
                return@forEachIndexed
            }
            val key = match.groupValues[1]
            val value = match.groupValues[2]
            if (key in catalog) errors += "${file.path}:${index + 1}: duplicate resource $key"
            if (value.isBlank()) errors += "${file.path}:${index + 1}: $key must not be blank"
            if (ALLOWED_PREFIXES.none(key::startsWith)) {
                errors += "${file.path}:${index + 1}: unsupported resource prefix for $key"
            }
            catalog[key] = value
        }
        if (catalog.keys.toList() != catalog.keys.sorted()) {
            errors += "${file.path}: resources must be alphabetically sorted"
        }
        return catalog
    }

    private fun checkShareCatalogs(
        english: Map<String, String>,
        french: Map<String, String>,
        errors: MutableList<String>,
    ) {
        val missing = english.keys - french.keys
        val extra = french.keys - english.keys
        if (missing.isNotEmpty()) errors += "iOS share fr: missing resources ${missing.sorted().joinToString()}"
        if (extra.isNotEmpty()) errors += "iOS share fr: unknown resources ${extra.sorted().joinToString()}"
        (english.keys intersect french.keys).forEach { key ->
            if (placeholders(english.getValue(key)) != placeholders(french.getValue(key))) {
                errors += "iOS share fr: placeholders differ for $key"
            }
        }
    }

    private fun checkSources(
        shareCatalogKeys: Set<String>,
        errors: MutableList<String>,
    ) {
        KotlinSourceChecker().use { kotlinChecker ->
            KOTLIN_SOURCE_SETS
                .flatMap { sourceSet -> composeAppDir.resolve("src/$sourceSet/kotlin").kotlinFiles() }
                .sortedBy(File::getPath)
                .forEach { file -> errors += kotlinChecker.check(file.path, file.readText()) }
        }

        val swiftChecker = SwiftSourceChecker(shareCatalogKeys)
        shareExtensionDir
            .walkTopDown()
            .filter { it.isFile && it.extension == "swift" }
            .sortedBy(File::getPath)
            .forEach { file -> errors += swiftChecker.check(file.path, file.readText()) }
    }

    private fun File.kotlinFiles(): List<File> =
        if (!isDirectory) emptyList() else walkTopDown().filter { it.isFile && it.extension == "kt" }.toList()

    private fun placeholders(value: String): List<String> =
        Regex("""%(?:\d+\$)?[A-Za-z@]""")
            .findAll(value)
            .map { it.value }
            .sorted()
            .toList()

    private companion object {
        val ALLOWED_PREFIXES =
            setOf(
                "auth_",
                "collections_",
                "common_",
                "feed_",
                "home_",
                "integrations_",
                "library_",
                "mila_",
                "nav_",
                "onboarding_",
                "prefs_",
                "profile_",
                "reader_",
                "search_",
                "share_",
                "sidebar_",
                "tags_",
                "trash_",
            )
        val KOTLIN_SOURCE_SETS = listOf("commonMain", "androidMain", "iosMain", "jvmMain")
    }
}
