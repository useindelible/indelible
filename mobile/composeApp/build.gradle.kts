import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.apple.XCFramework

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
        androidInstrumentedTest.dependencies {
            implementation(libs.androidx.testExt.junit)
            implementation(libs.androidx.test.runner)
            implementation(libs.kotlin.test)
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
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
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
    description = "Checks mobile localization catalogs and known user-visible source sinks."
    dependsOn(":i18n-checker:checkMobileI18n")
}

tasks.named("check") {
    dependsOn(i18nCheck)
}
