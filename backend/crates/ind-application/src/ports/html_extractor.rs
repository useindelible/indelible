#[derive(Debug, Clone)]
pub struct SpokenHtmlElement {
    pub tag: String,
    pub text: String,
}

pub trait HtmlExtractor: Send + Sync {
    fn extract_spoken_elements(&self, html: &str) -> Vec<SpokenHtmlElement>;
}
