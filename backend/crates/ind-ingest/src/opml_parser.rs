use quick_xml::Reader;
use quick_xml::events::Event;

use ind_application::ports::{OpmlParseError, OpmlParser};

pub struct QuickXmlOpmlParser;

impl QuickXmlOpmlParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QuickXmlOpmlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OpmlParser for QuickXmlOpmlParser {
    fn parse_feed_urls(&self, opml_xml: &str) -> Result<Vec<String>, OpmlParseError> {
        let mut reader = Reader::from_str(opml_xml);
        let mut urls = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"outline" {
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"xmlUrl"
                                && let Ok(val) = attr.decode_and_unescape_value(reader.decoder())
                            {
                                let url = val.trim().to_string();
                                if !url.is_empty() {
                                    urls.push(url);
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(OpmlParseError::Invalid(e.to_string())),
                _ => {}
            }
        }

        Ok(urls)
    }
}
