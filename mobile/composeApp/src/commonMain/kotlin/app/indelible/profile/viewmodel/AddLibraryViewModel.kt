package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.profile.repository.AddLibraryRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_add_url_error_invalid
import indelible.composeapp.generated.resources.library_add_url_error_submit
import io.ktor.http.parseUrl
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class AddLibraryUiState(
    val isSubmitting: Boolean = false,
    val errorMessage: UiMessage? = null,
)

sealed class AddLibraryEffect {
    data object Saved : AddLibraryEffect()
}

class AddLibraryViewModel(
    private val repository: AddLibraryRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow(AddLibraryUiState())
    val uiState: StateFlow<AddLibraryUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<AddLibraryEffect>()
    val effects: SharedFlow<AddLibraryEffect> = _effects.asSharedFlow()

    fun save(url: String) {
        if (_uiState.value.isSubmitting) return

        val normalizedUrl = url.trim()
        if (!isValidHttpUrl(normalizedUrl)) {
            _uiState.value = AddLibraryUiState(errorMessage = UiMessage(Res.string.library_add_url_error_invalid))
            return
        }

        _uiState.value = AddLibraryUiState(isSubmitting = true)
        viewModelScope.launch {
            repository
                .save(normalizedUrl)
                .onSuccess {
                    _uiState.value = AddLibraryUiState()
                    _effects.emit(AddLibraryEffect.Saved)
                }.onFailure {
                    _uiState.value =
                        AddLibraryUiState(
                            errorMessage = UiMessage(Res.string.library_add_url_error_submit),
                        )
                }
        }
    }

    fun clearError() {
        if (_uiState.value.errorMessage != null) {
            _uiState.value = _uiState.value.copy(errorMessage = null)
        }
    }

    fun reset() {
        if (!_uiState.value.isSubmitting) {
            _uiState.value = AddLibraryUiState()
        }
    }
}

private fun isValidHttpUrl(value: String): Boolean {
    if (value.isBlank() || value.any { it.isWhitespace() || it.code < 0x20 || it.code == 0x7f }) {
        return false
    }

    val url = runCatching { parseUrl(value) }.getOrNull() ?: return false
    val scheme = url.protocol.name
    return (scheme == "http" || scheme == "https") && isValidHost(url.host)
}

private fun isValidHost(host: String): Boolean {
    if (host.isBlank() || host.length > 253 || host.startsWith('.') || host.endsWith('.')) {
        return false
    }

    val normalizedHost =
        when {
            host.startsWith('[') && host.endsWith(']') -> host.substring(1, host.lastIndex)
            host.startsWith('[') || host.endsWith(']') -> return false
            else -> host
        }

    if (':' in normalizedHost) {
        return isValidIpv6(normalizedHost)
    }

    val labels = normalizedHost.split('.')
    if (labels.all { label -> label.isNotEmpty() && label.all(Char::isDigit) }) {
        return isValidIpv4(normalizedHost)
    }

    return labels.all { label ->
        label.length in 1..63 &&
            label.first().isLetterOrDigit() &&
            label.last().isLetterOrDigit() &&
            label.all { it.isLetterOrDigit() || it == '-' }
    }
}

private fun isValidIpv6(host: String): Boolean {
    val compressionIndex = host.indexOf("::")
    if (compressionIndex != host.lastIndexOf("::")) return false

    val hasCompression = compressionIndex >= 0
    val segments =
        if (hasCompression) {
            val left = host.substring(0, compressionIndex)
            val right = host.substring(compressionIndex + 2)
            val leftSegments = if (left.isEmpty()) emptyList() else left.split(':')
            val rightSegments = if (right.isEmpty()) emptyList() else right.split(':')
            leftSegments + rightSegments
        } else {
            if (host.startsWith(':') || host.endsWith(':')) return false
            host.split(':')
        }

    var groupCount = 0
    segments.forEachIndexed { index, segment ->
        if (segment.isEmpty()) return false
        if ('.' in segment) {
            if (index != segments.lastIndex || !isValidIpv4(segment)) return false
            groupCount += 2
        } else {
            if (segment.length !in 1..4 || !segment.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }) {
                return false
            }
            groupCount += 1
        }
    }

    return if (hasCompression) groupCount < 8 else groupCount == 8
}

private fun isValidIpv4(host: String): Boolean {
    val segments = host.split('.')
    return segments.size == 4 &&
        segments.all { segment ->
            segment.isNotEmpty() &&
                segment.length <= 3 &&
                segment.all(Char::isDigit) &&
                segment.toIntOrNull() in 0..255
        }
}
