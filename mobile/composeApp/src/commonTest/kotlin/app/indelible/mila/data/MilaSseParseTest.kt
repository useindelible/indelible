package app.indelible.mila.data

import kotlin.test.Test
import kotlin.test.assertEquals

class MilaSseParseTest {
    @Test
    fun non_data_lines_are_ignored() {
        assertEquals(SseLine.Ignore, parseSseLine("event: message"))
        assertEquals(SseLine.Ignore, parseSseLine(": heartbeat"))
        assertEquals(SseLine.Ignore, parseSseLine(""))
    }

    @Test
    fun blank_data_payload_is_ignored() {
        assertEquals(SseLine.Ignore, parseSseLine("data: "))
        assertEquals(SseLine.Ignore, parseSseLine("data:"))
    }

    @Test
    fun done_sentinel_maps_to_done() {
        assertEquals(SseLine.Done, parseSseLine("data: [DONE]"))
        assertEquals(SseLine.Done, parseSseLine("data:[DONE]"))
    }

    @Test
    fun delta_frame_maps_to_delta_text() {
        assertEquals(SseLine.Delta("Hello"), parseSseLine("""data: {"delta":"Hello"}"""))
        assertEquals(SseLine.Delta(" world"), parseSseLine("""data: {"delta":" world"}"""))
    }

    @Test
    fun error_frame_maps_to_error_message() {
        assertEquals(SseLine.Error("rate limited"), parseSseLine("""data: {"error":"rate limited"}"""))
    }

    @Test
    fun unparseable_error_frame_falls_back_to_a_generic_message() {
        val parsed = parseSseLine("""data: {"error":}""")
        assertEquals(SseLine.Error("Unknown stream error"), parsed)
    }

    @Test
    fun unparseable_delta_frame_is_ignored() {
        assertEquals(SseLine.Ignore, parseSseLine("""data: {"delta":}"""))
    }

    @Test
    fun unrecognized_json_payload_is_ignored() {
        assertEquals(SseLine.Ignore, parseSseLine("""data: {"foo":"bar"}"""))
    }
}
