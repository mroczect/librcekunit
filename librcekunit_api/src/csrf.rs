use librcekunit_handler::{Error, Result};
use scraper::{Html, Selector};

const TOKEN_SELECTOR: &str = "input[name='_token']";

pub fn extract(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let Ok(selector) = Selector::parse(TOKEN_SELECTOR) else {
        return Err(Error::CsrfNotFound);
    };
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("value").map(String::from))
        .ok_or(Error::CsrfNotFound)
}
