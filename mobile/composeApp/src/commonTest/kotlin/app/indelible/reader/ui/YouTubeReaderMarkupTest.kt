package app.indelible.reader.ui

import app.indelible.reader.model.ReaderPreferences
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Renders a YouTube reader body exactly as the worker emits it (see
 * backend/apps/ind-worker/src/jobs/youtube/html.rs) so the template's `.yt-*` rules are
 * exercised against real markup rather than assumed.
 */
class YouTubeReaderMarkupTest {
    private val youtubeBody =
        """
        <div class="yt-embed">
          <iframe width="560" height="315" src="https://www.youtube.com/embed/abc123" frameborder="0" allowfullscreen></iframe>
        </div>
        <div class="yt-channel-header">
          <div class="yt-channel-avatar">O</div>
          <div class="yt-channel-info">
            <span class="yt-channel-name">Ordinary Things</span>
            <div class="yt-video-stats">611.4K views<span class="yt-stat-dot"></span>44:31</div>
          </div>
        </div>
        <div class="yt-description">Want to restore the planet's ecosystems and see your impact in monthly videos? The first 100 people to join Planet Wild with my code ORDINARY6 will get the first month paid for by me.</div>
        <section class="yt-transcript">
          <h2>Transcript</h2>
          <div class="transcript-flow"><p><span class="t-seg" data-t="0:00">Good morning. How are you?</span> <span class="t-seg" data-t="0:14">There have been three themes running through the conference.</span></p>
        <p><span class="t-seg" data-t="2:18">Children starting school this year will be retiring in 2065.</span> <span class="t-seg" data-t="2:27">Nobody has a clue what the world will look like in five years.</span></p></div>
        </section>
        """.trimIndent()

    private fun render(isDark: Boolean) =
        ReaderHtmlTemplate.build(
            articleHtml = youtubeBody,
            preferences = ReaderPreferences(),
            highlights = emptyList(),
            isDarkMode = isDark,
            articleTitle = "How to Make Money",
            summaryHtml = "A tour of get-rich-quick schemes and why they do not work.",
        )

    @Test
    fun youtubeBodyIsStyledAndTimestampsAreTappable() {
        val html = render(isDark = false)

        assertTrue(html.contains(".yt-channel-header"), "channel header must be styled")
        assertTrue(html.contains(".yt-stat-dot"), "the stats separator must be styled or views/duration run together")
        assertTrue(html.contains(".yt-description"), "description must be styled")
        assertTrue(html.contains(".yt-transcript h2"), "transcript heading must be styled")
        assertTrue(html.contains(".t-seg::before"), "timestamp chip must be styled")
        assertTrue(html.contains("t-open"), "tap-to-reveal handler/state must be present")
        assertTrue(html.contains("How to Make Money"), "title comes from the masthead")
    }
}
