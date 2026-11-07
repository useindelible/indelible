package app.indelible.core.network

import app.indelible.core.offline.StaleWriteException
import kotlinx.io.IOException
import kotlinx.serialization.SerializationException
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ServerAnswerTest {
    @Test
    fun ioExceptionGaveNoAnswer() {
        assertTrue(IOException("connection reset").gaveNoAnswer())
    }

    @Test
    fun gatewayStatusesGaveNoAnswer() {
        listOf(502, 503, 504).forEach { assertTrue(ApiException(it, "gateway").gaveNoAnswer(), "status $it") }
    }

    @Test
    fun otherStatusesAreAnAnswer() {
        listOf(400, 404, 408, 429, 500).forEach { assertFalse(ApiException(it, "answer").gaveNoAnswer(), "status $it") }
    }

    @Test
    fun otherThrowablesAreAnAnswer() {
        assertFalse(SerializationException("bad body").gaveNoAnswer())
        assertFalse(StaleWriteException().gaveNoAnswer())
    }
}
