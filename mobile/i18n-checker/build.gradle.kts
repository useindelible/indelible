plugins {
    alias(libs.plugins.kotlinJvm)
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-compiler-embeddable:${libs.versions.kotlin.get()}")
    testImplementation(libs.kotlin.test)
}

tasks.test {
    useJUnitPlatform()
}

extensions.configure<io.gitlab.arturbosch.detekt.extensions.DetektExtension> {
    source.setFrom("src/main/kotlin", "src/test/kotlin")
}

val checkMobileI18n by tasks.registering(JavaExec::class) {
    group = "verification"
    description = "Checks mobile localization catalogs and known user-visible source sinks."
    dependsOn(tasks.test, tasks.named("detekt"))
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass = "app.indelible.i18ncheck.MobileI18nCheckerMainKt"

    val composeAppDir = project(":composeApp").layout.projectDirectory
    val shareExtensionDir = rootProject.layout.projectDirectory.dir("iosApp/IndelibleShareExtension")
    args(composeAppDir.asFile.absolutePath, shareExtensionDir.asFile.absolutePath)

    inputs.files(
        composeAppDir.dir("src/commonMain/composeResources").asFileTree,
        composeAppDir.dir("src/commonMain/kotlin").asFileTree,
        composeAppDir.dir("src/androidMain/kotlin").asFileTree,
        composeAppDir.dir("src/iosMain/kotlin").asFileTree,
        composeAppDir.dir("src/jvmMain/kotlin").asFileTree,
        shareExtensionDir.asFileTree,
    )
}
