use maud::{html, PreEscaped};

pub fn default_welcome(name: &str, version: &str, desc: Option<&str>, body: &str) -> String {
    let desc = desc.unwrap_or("");
    let desc_html = comrak::markdown_to_html(desc, &comrak::ComrakOptions::default());

    html! {
        h1 { "Hello, I am " (name) " version " (version) }
        p { "I'm running on <TODO insert carol host url here>" }
        (PreEscaped(desc_html))
        hr {}
        (PreEscaped(body))
    }
    .into_string()
}
