use maud::{html, PreEscaped};
use std::collections::BTreeMap;

use crate::activate::HttpMethod;

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

#[derive(Debug, Default)]
pub struct HttpCallList(
    BTreeMap<String, BTreeMap<HttpMethod, (Option<String>, Vec<(String, String)>)>>,
);

pub struct Call {
    pub path: String,
    pub http_method: HttpMethod,
    pub docs: Option<String>,
    pub params: Vec<(String, String)>,
    pub return_type: Option<String>,
}

impl HttpCallList {
    pub fn add_call(&mut self, call: Call) {
        let paths = self.0.entry(call.path).or_default();
        paths.insert(call.http_method, (call.docs, call.params));
    }

    pub fn render(&self) -> String {
        html! {
            @for (path, calls) in &self.0 {
                @for (method, (desc, params)) in calls {
                    @let desc_html = comrak::markdown_to_html(desc.as_deref().unwrap_or(""), &comrak::ComrakOptions::default());
                    h2 { (method) " " (path) }
                    p { (PreEscaped(desc_html)) }
                    h3 { "Paramters" }
                    ol {
                        @for (name, ty) in params {
                            li { (name) ": " code { (ty) } }
                        }
                    }
                }

            }
        }.into_string()
    }
}
