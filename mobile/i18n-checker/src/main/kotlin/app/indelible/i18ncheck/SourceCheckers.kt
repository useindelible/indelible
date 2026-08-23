package app.indelible.i18ncheck

import org.jetbrains.kotlin.K1Deprecation
import org.jetbrains.kotlin.cli.jvm.compiler.EnvironmentConfigFiles
import org.jetbrains.kotlin.cli.jvm.compiler.KotlinCoreEnvironment
import org.jetbrains.kotlin.com.intellij.openapi.util.Disposer
import org.jetbrains.kotlin.com.intellij.psi.PsiElement
import org.jetbrains.kotlin.com.intellij.psi.util.PsiTreeUtil
import org.jetbrains.kotlin.config.CommonConfigurationKeys
import org.jetbrains.kotlin.config.CompilerConfiguration
import org.jetbrains.kotlin.psi.KtAnnotated
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedDeclaration
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtPsiFactory
import org.jetbrains.kotlin.psi.KtStringTemplateExpression
import org.jetbrains.kotlin.psi.KtValueArgument

class KotlinSourceChecker : AutoCloseable {
    private val disposable = Disposer.newDisposable("mobile-i18n-check")
    private val psiFactory = createPsiFactory()

    fun check(
        fileName: String,
        source: String,
    ): List<String> {
        val file = psiFactory.createFile(fileName, source)
        return checkFile(fileName, source, file)
    }

    override fun close() = Disposer.dispose(disposable)

    private fun checkFile(
        fileName: String,
        source: String,
        file: KtFile,
    ): List<String> {
        val errors = mutableListOf<String>()
        val strings = PsiTreeUtil.collectElementsOfType(file, KtStringTemplateExpression::class.java)

        strings.forEach { expression ->
            val value = expression.literalText()
            val isRawVisibleCopy =
                value.any(Char::isLetter) &&
                    value !in BRAND_VALUES &&
                    expression.isVisible(file) &&
                    !expression.isIgnored(source)
            if (isRawVisibleCopy) {
                errors += "${expression.location(fileName, source)}: raw user-visible literal '$value'"
            }
        }

        MANUAL_PLURAL_PATTERNS.forEach { pattern ->
            pattern.findAll(source).forEach { match ->
                errors += "${location(fileName, source, match.range.first)}: manual English plural suffix"
            }
        }
        LOCALIZED_CASE_PATTERN.findAll(source).forEach { match ->
            errors += "${location(fileName, source, match.range.first)}: case transformation applied to localized text"
        }
        if (fileName in EMBEDDED_HTML_FILES) {
            VISIBLE_HTML_PATTERN.findAll(source).forEach { match ->
                if (!hasDurableIgnore(source, match.range.first)) {
                    errors += "${location(fileName, source, match.range.first)}: raw visible embedded HTML"
                }
            }
        }
        return errors.distinct()
    }

    @OptIn(K1Deprecation::class)
    private fun createPsiFactory(): KtPsiFactory {
        val configuration =
            CompilerConfiguration().apply {
                put(CommonConfigurationKeys.MODULE_NAME, "mobile-i18n-check")
                put(
                    CommonConfigurationKeys.MESSAGE_COLLECTOR_KEY,
                    org.jetbrains.kotlin.cli.common.messages.MessageCollector.NONE,
                )
            }

        @Suppress("DEPRECATION")
        val environment =
            KotlinCoreEnvironment.createForProduction(
                disposable,
                configuration,
                EnvironmentConfigFiles.JVM_CONFIG_FILES,
            )
        return KtPsiFactory(environment.project, markGenerated = false)
    }

    private fun KtStringTemplateExpression.literalText(): String =
        entries
            .filter { it.javaClass.simpleName in setOf("KtLiteralStringTemplateEntry", "KtEscapeStringTemplateEntry") }
            .joinToString("") { it.text }

    private fun KtStringTemplateExpression.isVisible(file: KtFile): Boolean =
        !isInPreview() &&
            (visibleValueArgument() != null || visibleProperty() != null || isLocallyAliasedVisible(file))

    private fun KtStringTemplateExpression.isLocallyAliasedVisible(file: KtFile): Boolean =
        (parent as? KtProperty)?.let { property ->
            property.name?.let { propertyName ->
                PsiTreeUtil
                    .collectElementsOfType(file, KtNameReferenceExpression::class.java)
                    .any { reference ->
                        reference.text == propertyName &&
                            reference.textRange != property.nameIdentifier?.textRange &&
                            (reference.visibleValueArgument() != null || reference.visibleProperty() != null)
                    }
            }
        } ?: false

    private fun PsiElement.visibleValueArgument(): KtValueArgument? {
        val candidate = this
        var current: PsiElement? = this
        while (current != null && current !is KtDeclaration) {
            if (current is KtValueArgument) {
                val argumentName = current.getArgumentName()?.asName?.identifier
                val call = parentCall(current)
                val callName = call?.calleeExpression?.text?.substringAfterLast('.')
                val isAnimationLabel =
                    argumentName == "label" &&
                        (callName.orEmpty().startsWith("animate") || callName == "rememberInfiniteTransition")
                val isDirectArgument = current.getArgumentExpression() == candidate
                val isVisibleSink = argumentName in VISIBLE_ARGUMENT_NAMES || callName in VISIBLE_CALLS
                if (isDirectArgument && !isAnimationLabel && isVisibleSink) {
                    return current
                }
            }
            current = current.parent
        }
        return null
    }

    private fun PsiElement.visibleProperty(): KtProperty? {
        var current: PsiElement? = this
        var visibleProperty: KtProperty? = null
        while (current != null && visibleProperty == null) {
            when {
                current is KtProperty && current.name in VISIBLE_PROPERTY_NAMES -> visibleProperty = current
                current is KtDeclaration && current !is KtProperty -> current = null
                else -> current = current.parent
            }
        }
        return visibleProperty
    }

    private fun PsiElement.isInPreview(): Boolean {
        var current: PsiElement? = this
        var isPreview = false
        while (current != null && !isPreview) {
            isPreview =
                current is KtAnnotated &&
                current.annotationEntries.any { it.shortName?.asString() == "Preview" } ||
                current is KtNamedFunction &&
                current.hasPreviewFixtureName() ||
                current is KtProperty &&
                current.hasPreviewFixtureName()
            current = current.parent
        }
        return isPreview
    }

    private fun PsiElement.isIgnored(source: String): Boolean = hasDurableIgnore(source, textRange.startOffset)

    private fun KtNamedDeclaration.hasPreviewFixtureName(): Boolean =
        name?.let { it.startsWith("preview") || it.startsWith("sample") } == true

    private fun PsiElement.location(
        fileName: String,
        source: String,
    ): String = location(fileName, source, textRange.startOffset)

    private companion object {
        val BRAND_VALUES =
            setOf("DELETE", "Indelible", "Mila", "MILA", "Ollama", "OpenAI", "RSS", "OPML", "PDF", "EPUB")
        val EMBEDDED_HTML_FILES = setOf("ReaderHtmlMarkup.kt", "ReaderHtmlTemplate.kt")
        val VISIBLE_ARGUMENT_NAMES =
            setOf(
                "contentDescription",
                "error",
                "eyebrow",
                "hint",
                "label",
                "message",
                "onClickLabel",
                "subtitle",
                "text",
                "title",
            )
        val VISIBLE_CALLS = setOf("BasicText", "ShowSnackbar", "Text", "showSnackbar")
        val VISIBLE_PROPERTY_NAMES = setOf("description", "displayName")
        val MANUAL_PLURAL_PATTERNS =
            listOf(
                Regex("""if\s*\([^)]*\)\s*\\?"s\\?"\s*else\s*\\?"\\?"""),
                Regex("""if\s*\([^)]*\)\s*\\?"\\?"\s*else\s*\\?"s\\?"""),
            )
        val LOCALIZED_CASE_PATTERN =
            Regex(
                """(?:stringResource|pluralStringResource|resolve)\([^)]*\)""" +
                    """\s*\.(?:lowercase|uppercase|capitalize)\(""",
            )
        val VISIBLE_HTML_PATTERN = Regex(""">\s*[A-Za-z][^<\n]{1,}<""")
    }
}

class SwiftSourceChecker(
    private val catalogKeys: Set<String>,
) {
    fun check(
        fileName: String,
        source: String,
    ): List<String> {
        val errors = mutableListOf<String>()
        val code = maskSwiftComments(source)
        VISIBLE_PATTERNS.forEach { pattern ->
            pattern.findAll(code).forEach { match ->
                val value = match.groupValues[1]
                if (value !in catalogKeys && value !in BRAND_VALUES && !hasDurableIgnore(source, match.range.first)) {
                    errors +=
                        "${location(fileName, source, match.range.first)}: " +
                        "raw Swift user-visible literal '$value'"
                }
            }
        }
        return errors
    }

    private companion object {
        val BRAND_VALUES = setOf("Indelible", "Mila", "Ollama", "OpenAI", "RSS", "OPML", "PDF", "EPUB")
        val VISIBLE_PATTERNS =
            listOf(
                Regex("""\b(?:Text|Button)\(\s*"([^"]+)"""),
                Regex("""\.accessibilityLabel\(\s*"([^"]+)"""),
                Regex("""\bStatusRow\(\s*key:\s*"([^"]+)"""),
            )
    }
}

private fun maskSwiftComments(source: String): String = SwiftCommentMasker(source).mask()

private class SwiftCommentMasker(
    private val source: String,
) {
    private val masked = source.toCharArray()
    private var index = 0
    private var state = State.CODE
    private var blockDepth = 0

    fun mask(): String {
        while (index < source.length) {
            when (state) {
                State.CODE -> visitCode()
                State.STRING -> visitString()
                State.LINE_COMMENT -> visitLineComment()
                State.BLOCK_COMMENT -> visitBlockComment()
            }
        }
        return masked.concatToString()
    }

    private fun visitCode() {
        when {
            source.startsWith("//", index) -> startComment(State.LINE_COMMENT)
            source.startsWith("/*", index) -> {
                blockDepth = 1
                startComment(State.BLOCK_COMMENT)
            }
            source[index] == '"' -> {
                state = State.STRING
                index++
            }
            else -> index++
        }
    }

    private fun visitString() {
        if (source[index] == '"' && (index == 0 || source[index - 1] != '\\')) state = State.CODE
        index++
    }

    private fun visitLineComment() {
        if (source[index] == '\n') {
            state = State.CODE
        } else {
            masked[index] = ' '
        }
        index++
    }

    private fun visitBlockComment() {
        when {
            source.startsWith("/*", index) -> {
                blockDepth++
                maskPair()
            }
            source.startsWith("*/", index) -> {
                blockDepth--
                maskPair()
                if (blockDepth == 0) state = State.CODE
            }
            else -> {
                if (source[index] != '\n') masked[index] = ' '
                index++
            }
        }
    }

    private fun startComment(newState: State) {
        state = newState
        maskPair()
    }

    private fun maskPair() {
        masked[index] = ' '
        masked[index + 1] = ' '
        index += 2
    }

    private enum class State {
        BLOCK_COMMENT,
        CODE,
        LINE_COMMENT,
        STRING,
    }
}

private fun parentCall(element: PsiElement): KtCallExpression? {
    var current = element.parent
    while (current != null) {
        if (current is KtCallExpression) return current
        current = current.parent
    }
    return null
}

private fun hasDurableIgnore(
    source: String,
    offset: Int,
): Boolean {
    val lineStart = source.lastIndexOf('\n', offset.coerceAtMost(source.length - 1)).let { if (it == -1) 0 else it + 1 }
    val lineEnd = source.indexOf('\n', offset).let { if (it == -1) source.length else it }
    val marker = Regex("""//\s*i18n-ignore:\s*(.+)$""").find(source.substring(lineStart, lineEnd)) ?: return false
    return marker.groupValues[1].trim().length >= MINIMUM_IGNORE_REASON_LENGTH
}

private fun location(
    fileName: String,
    source: String,
    offset: Int,
): String = "$fileName:${source.take(offset).count { it == '\n' } + 1}"

private const val MINIMUM_IGNORE_REASON_LENGTH = 8
