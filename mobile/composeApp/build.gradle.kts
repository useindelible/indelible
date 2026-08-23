import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.apple.XCFramework
import org.w3c.dom.Element
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
    alias(libs.plugins.kotlinSerialization)
    alias(libs.plugins.fabrikt)
}

fabrikt {
    generate("indelibleApi") {
        apiFile = file("$rootDir/openapi-spec.json")
        basePackage = "app.indelible.api.generated"
        outputDirectory = file("${layout.buildDirectory.get()}/generated/fabrikt")
        sourcesPath = "src/commonMain/kotlin"
        resourcesPath = "src/commonMain/resources"
        validationLibrary = NoValidation
        model {
            generate = enabled
            serializationLibrary = Kotlinx
            instantLibrary = Kotlinx
            ignoreUnknownProperties = disabled
        }
        client {
            generate = enabled
            target = Ktor
        }
        controller {
            generate = disabled
        }
        typeOverrides {
            uuid = String
        }
    }
}

val fabriktAnyPatchTargets: List<Pair<java.io.File, List<Pair<String, String>>>> = listOf(
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/IntegrationConnectionDto.kt",
    ).asFile to listOf("public val config: Any," to "public val config: kotlinx.serialization.json.JsonElement,"),
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/CreateSmartListBody.kt",
    ).asFile to listOf("public val filterExpression: Any," to "public val filterExpression: kotlinx.serialization.json.JsonElement,"),
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/UpdateSmartListBody.kt",
    ).asFile to listOf("public val filterExpression: Any? = null," to "public val filterExpression: kotlinx.serialization.json.JsonElement? = null,"),
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/SmartListResponse.kt",
    ).asFile to listOf("public val filterExpression: Any," to "public val filterExpression: kotlinx.serialization.json.JsonElement,"),
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/LibraryQueryBody.kt",
    ).asFile to listOf("public val filterExpression: Any? = null," to "public val filterExpression: kotlinx.serialization.json.JsonElement? = null,"),
    layout.projectDirectory.file(
        "build/generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/models/MilaAiOutputResponse.kt",
    ).asFile to listOf("public val content: Any," to "public val content: kotlinx.serialization.json.JsonElement,"),
)
val fabriktGeneratedClientDir =
    layout.buildDirectory
        .dir("generated/fabrikt/src/commonMain/kotlin/app/indelible/api/generated/client")
        .get()
        .asFile
val fabriktGeneratedKtorModels =
    layout.buildDirectory
        .file("generated/fabrikt/src/main/kotlin/app/indelible/api/generated/client/KtorApiModels.kt")
        .get()
        .asFile
tasks.register("patchFabriktGenerated") {
    group = "codegen"
    description = "Normalize Fabrikt DTO, common I/O, and query-parameter output for Kotlin Multiplatform."
    dependsOn("fabriktGenerate")
    val targets = fabriktAnyPatchTargets
    val generatedClientDir = fabriktGeneratedClientDir
    val generatedKtorModels = fabriktGeneratedKtorModels
    val queryParameterPattern = Regex("""add\("([^"]+)=\$\{([^}]+)}"\)""")
    doLast {
        targets.forEach { (file, replacements) ->
            if (!file.exists()) return@forEach
            var patched = file.readText().replace("import kotlin.Any\n", "")
            replacements.forEach { (from, to) ->
                patched = patched.replace(from, to)
            }
            file.writeText(patched)
        }
        // Fabrikt's Ktor templates emit JVM IOException imports and place two support types under
        // src/main. Normalize those generated files so the client remains valid commonMain code.
        generatedClientDir
            .walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .forEach { file ->
                val source =
                    file
                        .readText()
                        .replace("import java.io.IOException", "import kotlinx.io.IOException")
                        .replace(
                            "import io.ktor.utils.io.errors.IOException",
                            "import kotlinx.io.IOException",
                        )
                val encoded =
                    queryParameterPattern.replace(source) { match ->
                        val name = match.groupValues[1]
                        val expression = match.groupValues[2]
                        if (expression.endsWith(".encodeQueryParameter()")) {
                            match.value
                        } else {
                            "add(\"$name=\${${expression}.encodeQueryParameter()}\")"
                        }
                    }
                val patched =
                    if (encoded != source && "import io.ktor.http.encodeURLQueryComponent" !in encoded) {
                        encoded
                            .replaceFirst(
                                "\n\n",
                                "\n\nimport io.ktor.http.encodeURLQueryComponent\n",
                            ).replaceFirst(
                                "public class ",
                                "private fun Any?.encodeQueryParameter(): String =\n" +
                                    "  toString().encodeURLQueryComponent(encodeFull = true)\n\n" +
                                    "public class ",
                            )
                    } else {
                        encoded
                    }
                file.writeText(
                    patched,
                )
            }
        generatedKtorModels
            .takeIf { it.exists() }
            ?.let { file ->
                file.writeText(
                    file
                        .readText()
                        .replace("import java.io.IOException", "import kotlinx.io.IOException")
                        .replace(
                            "import io.ktor.utils.io.errors.IOException",
                            "import kotlinx.io.IOException",
                        ),
                )
            }
    }
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.AbstractKotlinCompile<*>>().configureEach {
    dependsOn("patchFabriktGenerated")
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinNativeCompile>().configureEach {
    dependsOn("patchFabriktGenerated")
}

tasks.matching { it.name.startsWith("runKtlint") }.configureEach {
    dependsOn("patchFabriktGenerated")
}

tasks.matching { it.name == "detekt" }.configureEach {
    dependsOn("patchFabriktGenerated")
}

tasks.register<Exec>("exportOpenApiSpec") {
    group = "codegen"
    description = "Export OpenAPI spec from backend"
    workingDir = rootDir.parentFile.resolve("backend")
    commandLine(
        "cargo",
        "run",
        "-p",
        "ind-http-api",
        "--example",
        "export_openapi",
        "--",
        rootDir.resolve("openapi-spec.json").absolutePath,
    )
}

// CI bakes the Cloud server URL with -Pindelible.serverUrlDefault=...; OSS/store
// builds without it ask for a self-hosted address on first launch. Local dev can
// set -Pindelible.devServerPrefill=http://localhost:38473 to prefill the field.
val serverUrlDefault = (findProperty("indelible.serverUrlDefault") as String?)?.trim().orEmpty()
val devServerPrefill = (findProperty("indelible.devServerPrefill") as String?)?.trim().orEmpty()
val generateServerBuildConfig =
    tasks.register("generateServerBuildConfig") {
        group = "codegen"
        description = "Generate ServerBuildConfig from Gradle properties"
        // Locals shadow the script-level values so the doLast closure stays
        // configuration-cache serializable (no script object capture).
        val urlDefault = serverUrlDefault
        val devPrefill = devServerPrefill
        val outputDir = layout.buildDirectory.dir("generated/serverconfig/kotlin")
        inputs.property("serverUrlDefault", urlDefault)
        inputs.property("devServerPrefill", devPrefill)
        outputs.dir(outputDir)
        doLast {
            val target = outputDir.get().file("app/indelible/core/config/ServerBuildConfig.kt").asFile
            target.parentFile.mkdirs()
            target.writeText(
                """
                package app.indelible.core.config

                object ServerBuildConfig {
                    const val SERVER_URL_DEFAULT: String = "$urlDefault"
                    const val DEV_SERVER_PREFILL: String = "$devPrefill"
                }
                """.trimIndent() + "\n",
            )
        }
    }

kotlin {
    jvmToolchain(21)

    androidTarget {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_21)
        }
    }

    val xcf = XCFramework("ComposeApp")
    listOf(
        iosArm64(),
        iosSimulatorArm64(),
    ).forEach { iosTarget ->
        iosTarget.binaries.framework {
            baseName = "ComposeApp"
            isStatic = true
            xcf.add(this)
        }
    }

    jvm()

    sourceSets.commonMain {
        kotlin.srcDir("${layout.buildDirectory.get()}/generated/fabrikt/src/commonMain/kotlin")
        kotlin.srcDir("${layout.buildDirectory.get()}/generated/fabrikt/src/main/kotlin")
        kotlin.srcDir(generateServerBuildConfig)
    }

    sourceSets {
        androidMain.dependencies {
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.activity.compose)
            implementation(libs.androidx.security.crypto)
            implementation(libs.androidx.splashscreen)
            implementation(libs.androidx.browser)
            implementation(libs.ktor.client.okhttp)
        }
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(compose.materialIconsExtended)
            implementation(compose.components.resources)
            implementation(libs.compose.ui)
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)
            implementation(libs.androidx.navigation.compose)
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.datetime)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.ktor.client.core)
            implementation(libs.ktor.client.content.negotiation)
            implementation(libs.ktor.client.auth)
            implementation(libs.ktor.serialization.kotlinx.json)
            implementation(libs.coil.compose)
            implementation(libs.coil.network.ktor)
            implementation(libs.koin.core)
            implementation(libs.markdown.renderer.m3)
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
            implementation(libs.kotlinx.coroutines.test)
            implementation(libs.kotlinx.datetime)
            implementation(libs.ktor.client.mock)
            implementation(libs.compose.ui.test)
        }
        iosMain.dependencies {
            implementation(libs.ktor.client.darwin)
        }
        jvmMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)
            implementation(libs.ktor.client.cio)
        }
        jvmTest.dependencies {
            implementation(compose.desktop.currentOs)
        }
    }
}

android {
    namespace = "app.indelible"
    compileSdk =
        libs.versions.android.compileSdk
            .get()
            .toInt()

    defaultConfig {
        // Store identity (reverse-DNS of useindelible.com); the Kotlin source
        // namespace stays app.indelible — the two are independent by design.
        applicationId = "com.useindelible.app"
        minSdk =
            libs.versions.android.minSdk
                .get()
                .toInt()
        targetSdk =
            libs.versions.android.targetSdk
                .get()
                .toInt()
        versionCode = 1
        versionName = "0.1.0"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    // CI decodes the keystore from secrets and points these variables at it.
    // Absent locally, so debug builds and local assembleRelease are unaffected
    // (local release output stays unsigned).
    val keystorePath = System.getenv("ANDROID_KEYSTORE_PATH")
    signingConfigs {
        if (keystorePath != null) {
            create("release") {
                storeFile = file(keystorePath)
                storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("ANDROID_KEY_ALIAS")
                keyPassword = System.getenv("ANDROID_KEY_PASSWORD")
            }
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            if (keystorePath != null) {
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }
}

dependencies {
    debugImplementation(libs.compose.uiTooling)
}

compose.desktop {
    application {
        mainClass = "app.indelible.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "app.indelible"
            packageVersion = "1.0.0"
        }
    }
}

val i18nCheck by tasks.registering {
    group = "verification"
    description = "Checks mobile localization catalogs and production source coverage."

    val resourcesRoot = layout.projectDirectory.dir("src/commonMain/composeResources")
    val sourceFile = resourcesRoot.file("values/strings.xml")
    val translationFiles =
        resourcesRoot.asFileTree.matching {
            include("values-*/strings.xml")
        }
    val kotlinSourceFiles =
        files(
            fileTree(layout.projectDirectory.dir("src/commonMain/kotlin")) { include("**/*.kt") },
            fileTree(layout.projectDirectory.dir("src/androidMain/kotlin")) { include("**/*.kt") },
        )
    val shareExtensionRoot = rootProject.layout.projectDirectory.dir("iosApp/IndelibleShareExtension")
    val shareEnglishFile = shareExtensionRoot.file("en.lproj/Localizable.strings")
    val shareFrenchFile = shareExtensionRoot.file("fr.lproj/Localizable.strings")
    val shareSwiftFiles = fileTree(shareExtensionRoot) { include("**/*.swift") }
    inputs.file(sourceFile)
    inputs.files(translationFiles)
    inputs.files(kotlinSourceFiles)
    inputs.file(shareEnglishFile)
    inputs.file(shareFrenchFile)
    inputs.files(shareSwiftFiles)

    doLast {
        val errors = mutableListOf<String>()
        val allowedPrefixes =
            setOf(
                "common_",
                "auth_",
                "onboarding_",
                "nav_",
                "home_",
                "library_",
                "sidebar_",
                "feed_",
                "search_",
                "collections_",
                "tags_",
                "trash_",
                "profile_",
                "prefs_",
                "integrations_",
                "reader_",
                "mila_",
                "share_",
            )

        fun readCatalog(file: File): Map<String, Pair<String, Map<String, String>>> {
            val document = DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(file)
            val catalog = linkedMapOf<String, Pair<String, Map<String, String>>>()
            val children = document.documentElement.childNodes

            for (index in 0 until children.length) {
                val element = children.item(index) as? Element ?: continue
                if (element.tagName !in setOf("string", "plurals")) continue
                val name = element.getAttribute("name")
                if (name.isBlank()) errors += "${file.path}: resource name must not be empty"
                if (name in catalog) errors += "${file.path}: duplicate resource $name"
                if (allowedPrefixes.none(name::startsWith)) {
                    errors += "${file.path}: unsupported resource prefix for $name"
                }

                val values = linkedMapOf<String, String>()
                if (element.tagName == "string") {
                    values["value"] = element.textContent.trim()
                } else {
                    val items = element.childNodes
                    for (itemIndex in 0 until items.length) {
                        val item = items.item(itemIndex) as? Element ?: continue
                        if (item.tagName != "item") continue
                        val quantity = item.getAttribute("quantity")
                        if (quantity in values) {
                            errors += "${file.path}: duplicate $name quantity $quantity"
                        }
                        values[quantity] = item.textContent.trim()
                    }
                }
                if (values.isEmpty() || values.values.any(String::isBlank)) {
                    errors += "${file.path}: $name must not contain empty values"
                }
                catalog[name] = element.tagName to values
            }

            val keys = catalog.keys.toList()
            if (keys != keys.sorted()) {
                val firstMismatch = keys.zip(keys.sorted()).first { (actual, expected) -> actual != expected }
                errors += "${file.path}: resources must be alphabetically sorted; found ${firstMismatch.first} before ${firstMismatch.second}"
            }

            return catalog
        }

        fun placeholders(value: String): List<String> =
            Regex("""%(?:\d+\$)?[A-Za-z@]""").findAll(value).map { it.value }.sorted().toList()

        val source = readCatalog(sourceFile.asFile)
        translationFiles.files.sortedBy(File::getPath).forEach { file ->
            val translation = readCatalog(file)
            val locale = file.parentFile.name.removePrefix("values-")
            val extra = translation.keys - source.keys
            if (extra.isNotEmpty()) errors += "$locale: unknown resources ${extra.sorted().joinToString()}"
            if (locale == "fr") {
                val missing = source.keys - translation.keys
                if (missing.isNotEmpty()) errors += "fr: missing resources ${missing.sorted().joinToString()}"
            }

            (source.keys intersect translation.keys).forEach { key ->
                val (sourceType, sourceValues) = source.getValue(key)
                val (translatedType, translatedValues) = translation.getValue(key)
                if (sourceType != translatedType) {
                    errors += "$locale: resource type differs for $key"
                }
                if (sourceValues.keys != translatedValues.keys) {
                    errors += "$locale: plural quantities differ for $key"
                }
                (sourceValues.keys intersect translatedValues.keys).forEach { quantity ->
                    if (placeholders(sourceValues.getValue(quantity)) != placeholders(translatedValues.getValue(quantity))) {
                        errors += "$locale: placeholders differ for $key ($quantity)"
                    }
                }
            }
        }

        fun readStringsCatalog(file: File): Map<String, String> {
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
                if (allowedPrefixes.none(key::startsWith)) {
                    errors += "${file.path}:${index + 1}: unsupported resource prefix for $key"
                }
                catalog[key] = value
            }
            if (catalog.keys.toList() != catalog.keys.sorted()) {
                errors += "${file.path}: resources must be alphabetically sorted"
            }
            return catalog
        }

        val shareEnglish = readStringsCatalog(shareEnglishFile.asFile)
        val shareFrench = readStringsCatalog(shareFrenchFile.asFile)
        val missingShareKeys = shareEnglish.keys - shareFrench.keys
        val extraShareKeys = shareFrench.keys - shareEnglish.keys
        if (missingShareKeys.isNotEmpty()) errors += "iOS share fr: missing resources ${missingShareKeys.sorted().joinToString()}"
        if (extraShareKeys.isNotEmpty()) errors += "iOS share fr: unknown resources ${extraShareKeys.sorted().joinToString()}"
        (shareEnglish.keys intersect shareFrench.keys).forEach { key ->
            if (placeholders(shareEnglish.getValue(key)) != placeholders(shareFrench.getValue(key))) {
                errors += "iOS share fr: placeholders differ for $key"
            }
        }

        fun maskComments(sourceText: String): String {
            val output = sourceText.toCharArray()
            var index = 0
            var inString = false
            var inChar = false
            var inTripleString = false
            var inBlockComment = false
            while (index < sourceText.length) {
                if (inBlockComment) {
                    if (sourceText.startsWith("*/", index)) {
                        output[index] = ' '
                        output[index + 1] = ' '
                        index += 2
                        inBlockComment = false
                    } else {
                        if (output[index] != '\n') output[index] = ' '
                        index++
                    }
                    continue
                }
                if (!inString && !inChar && !inTripleString && sourceText.startsWith("//", index)) {
                    while (index < sourceText.length && sourceText[index] != '\n') output[index++] = ' '
                    continue
                }
                if (!inString && !inChar && !inTripleString && sourceText.startsWith("/*", index)) {
                    output[index] = ' '
                    output[index + 1] = ' '
                    index += 2
                    inBlockComment = true
                    continue
                }
                if (!inString && !inChar && sourceText.startsWith("\"\"\"", index)) {
                    inTripleString = !inTripleString
                    index += 3
                    continue
                }
                if (!inTripleString && !inChar && sourceText[index] == '"' && (index == 0 || sourceText[index - 1] != '\\')) {
                    inString = !inString
                } else if (!inTripleString && !inString && sourceText[index] == '\'' &&
                    (index == 0 || sourceText[index - 1] != '\\')
                ) {
                    inChar = !inChar
                }
                index++
            }
            return output.concatToString()
        }

        fun maskPreviewBodies(sourceText: String): String {
            val output = sourceText.toCharArray()
            Regex("""@(?:[A-Za-z0-9_.]+\.)?Preview\b""").findAll(sourceText).forEach { preview ->
                val functionStart = sourceText.indexOf("fun ", preview.range.last + 1)
                if (functionStart == -1) return@forEach
                val bodyStart = sourceText.indexOf('{', functionStart)
                if (bodyStart == -1) return@forEach
                var depth = 0
                var bodyEnd = -1
                for (index in bodyStart until sourceText.length) {
                    when (sourceText[index]) {
                        '{' -> depth++
                        '}' -> {
                            depth--
                            if (depth == 0) {
                                bodyEnd = index
                                break
                            }
                        }
                    }
                }
                if (bodyEnd != -1) {
                    for (index in preview.range.first..bodyEnd) {
                        if (output[index] != '\n') output[index] = ' '
                    }
                }
            }
            return output.concatToString()
        }

        fun maskPreviewFixtures(sourceText: String): String {
            if ("Preview" !in sourceText) return sourceText
            val output = sourceText.toCharArray()
            val fixturePattern = Regex("""(?m)^private\s+(?:fun|val)\s+(?:preview|sample)[A-Za-z0-9_]*""")
            val topLevelDeclaration = Regex("""(?m)^(?:@|private\s|internal\s|public\s|fun\s|class\s|object\s)""")
            fixturePattern.findAll(sourceText).forEach { fixture ->
                val nextDeclaration =
                    topLevelDeclaration.find(sourceText, fixture.range.last + 1)?.range?.first ?: sourceText.length
                for (index in fixture.range.first until nextDeclaration) {
                    if (output[index] != '\n') output[index] = ' '
                }
            }
            return output.concatToString()
        }

        val kotlinSinkPatterns =
            listOf(
                Regex("""\b(?:Text|BasicText)\s*\(\s*"([^"]*[A-Za-z][^"]*)"""),
                Regex("""\b(?:text|title|subtitle|label|hint|message|eyebrow|contentDescription|onClickLabel|error)\s*=\s*"([^"]*[A-Za-z][^"]*)"""),
                Regex("""\b(?:showSnackbar|ShowSnackbar)\s*\(\s*"([^"]*[A-Za-z][^"]*)"""),
                Regex("""\b(?:displayName|description)\s*(?:=|get\(\)\s*=)\s*"([^"]*[A-Za-z][^"]*)"""),
            )
        val brandValues = setOf("Indelible", "Mila", "MILA", "Ollama", "OpenAI", "RSS", "OPML", "PDF", "EPUB")

        fun hasDurableIgnore(originalLine: String): Boolean {
            val marker = Regex("""//\s*i18n-ignore:\s*(.+)$""").find(originalLine) ?: return false
            if (marker.groupValues[1].trim().length < 8) {
                errors += "i18n-ignore reason is too short: ${originalLine.trim()}"
            }
            return true
        }

        fun lineNumberAt(
            sourceText: String,
            offset: Int,
        ): Int = sourceText.take(offset).count { it == '\n' } + 1

        kotlinSourceFiles.files.sortedBy(File::getPath).forEach { file ->
            val original = file.readText()
            val masked = maskPreviewBodies(maskPreviewFixtures(maskComments(original)))
            val originalLines = original.lines()
            kotlinSinkPatterns.forEach { pattern ->
                pattern.findAll(masked).forEach matchLoop@{ match ->
                    val literalOffset = match.range.first + match.value.indexOf('"')
                    val lineNumber = lineNumberAt(masked, literalOffset)
                    if (hasDurableIgnore(originalLines[lineNumber - 1])) return@matchLoop
                    val value = match.groupValues[1]
                    val staticValue = value.replace(Regex("""\$\{[^}]+}|\$[A-Za-z_][A-Za-z0-9_]*"""), "")
                    val prefix = masked.take(match.range.first).takeLast(500)
                    val animationLabel =
                        "label" in match.value &&
                            listOf("animate", "rememberInfiniteTransition").any(prefix::contains)
                    if (!animationLabel && staticValue.any(Char::isLetter) && value !in brandValues) {
                        errors += "${file.path}:$lineNumber: raw user-visible literal '$value'"
                    }
                }
            }
            Regex("""if\s*\([^)]*\)\s*"s"\s*else\s*""""").findAll(masked).forEach { match ->
                errors += "${file.path}:${lineNumberAt(masked, match.range.first)}: manual English plural suffix"
            }
            Regex("""(?:stringResource|pluralStringResource|resolve)\([^)]*\)\s*\.(?:lowercase|uppercase|capitalize)\(""")
                .findAll(masked)
                .forEach { match ->
                    errors +=
                        "${file.path}:${lineNumberAt(masked, match.range.first)}: " +
                        "case transformation applied to localized text"
                }
            if (file.name in setOf("ReaderHtmlMarkup.kt", "ReaderHtmlTemplate.kt")) {
                Regex(""">\s*[A-Za-z][^<\n]{1,}<""").findAll(masked).forEach { match ->
                    val lineNumber = lineNumberAt(masked, match.range.first)
                    if (!hasDurableIgnore(originalLines[lineNumber - 1])) {
                        errors += "${file.path}:$lineNumber: raw visible embedded HTML"
                    }
                }
            }
        }

        val swiftVisiblePatterns =
            listOf(
                Regex("""\b(?:Text|Button)\(\s*"([^"]+)"""),
                Regex("""\.accessibilityLabel\(\s*"([^"]+)"""),
            )
        shareSwiftFiles.files.sortedBy(File::getPath).forEach { file ->
            val original = file.readText()
            val masked = maskComments(original)
            val originalLines = original.lines()
            swiftVisiblePatterns.forEach { pattern ->
                pattern.findAll(masked).forEach matchLoop@{ match ->
                    val literalOffset = match.range.first + match.value.indexOf('"')
                    val lineNumber = lineNumberAt(masked, literalOffset)
                    if (hasDurableIgnore(originalLines[lineNumber - 1])) return@matchLoop
                    val value = match.groupValues[1]
                    if (value !in shareEnglish && value !in brandValues) {
                        errors += "${file.path}:$lineNumber: raw Swift user-visible literal '$value'"
                    }
                }
            }
        }

        if (errors.isNotEmpty()) {
            throw GradleException(errors.joinToString(prefix = "Mobile i18n check failed:\n- ", separator = "\n- "))
        }
        logger.lifecycle(
            "Mobile i18n check passed ({} Compose resources, {} translations, {} iOS share resources)",
            source.size,
            translationFiles.files.size,
            shareEnglish.size,
        )
    }
}

tasks.named("check") {
    dependsOn(i18nCheck)
}
