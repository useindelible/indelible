package app.indelible.auth.server

import io.ktor.http.parseUrl

sealed interface ServerUrlValidation {
    data class Invalid(
        val message: String,
    ) : ServerUrlValidation

    data class NeedsCleartextConsent(
        val url: String,
    ) : ServerUrlValidation

    data class Ready(
        val url: String,
    ) : ServerUrlValidation
}

object ServerUrlForm {
    const val EMPTY_ADDRESS_MESSAGE = "Enter your server address."
    const val INVALID_ADDRESS_MESSAGE = "That doesn't look like a valid address."
    const val SCHEME_STUB = "https://"

    // Mirrors the loopback exceptions in androidMain res/xml/network_security_config.xml:
    // these hosts never leave the device, so the cleartext consent gate is skipped.
    private val LOOPBACK_HOSTS = setOf("localhost", "127.0.0.1", "10.0.2.2", "10.0.3.2")

    private val SCHEME_REGEX = Regex("^[a-zA-Z][a-zA-Z0-9+.-]*://")

    fun normalize(input: String): String {
        var candidate = input.trim()
        if (candidate.isEmpty()) return candidate
        // Defense against doubled schemes from edits or pastes ("https://http://host"):
        // strip the leading https:// while another scheme follows it.
        while (candidate.startsWith(SCHEME_STUB) &&
            SCHEME_REGEX.containsMatchIn(candidate.substring(SCHEME_STUB.length))
        ) {
            candidate = candidate.substring(SCHEME_STUB.length)
        }
        val withScheme = if (SCHEME_REGEX.containsMatchIn(candidate)) candidate else SCHEME_STUB + candidate
        return withScheme.trimEnd('/')
    }

    fun validate(input: String): ServerUrlValidation {
        val normalized = normalize(input)
        if (normalized.isEmpty()) return ServerUrlValidation.Invalid(EMPTY_ADDRESS_MESSAGE)
        val url = parseUrl(normalized) ?: return ServerUrlValidation.Invalid(INVALID_ADDRESS_MESSAGE)
        val scheme = url.protocol.name
        if (scheme != "http" && scheme != "https") {
            return ServerUrlValidation.Invalid(INVALID_ADDRESS_MESSAGE)
        }
        val host = url.host
        if (host.isBlank() || host.any { it.isWhitespace() }) {
            return ServerUrlValidation.Invalid(INVALID_ADDRESS_MESSAGE)
        }
        if (scheme == "http" && host !in LOOPBACK_HOSTS) {
            return ServerUrlValidation.NeedsCleartextConsent(normalized)
        }
        return ServerUrlValidation.Ready(normalized)
    }

    fun displayHost(url: String): String {
        val parsed = parseUrl(normalize(url)) ?: return url
        return parsed.host.ifBlank { url }
    }
}
