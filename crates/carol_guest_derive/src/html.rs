use maud::{html, PreEscaped, DOCTYPE};

fn var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .and_then(|val| if val.is_empty() { None } else { Some(val) })
}

pub fn default_welcome(desc: Option<&str>, body: &str) -> String {
    let desc_html = comrak::markdown_to_html(desc.unwrap_or(""), &comrak::ComrakOptions::default());

    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title {
            @if let Some(pkg_name) = var("CARGO_PKG_NAME") {
                (pkg_name)
                @if let Some(pkg_version) = var("CARGO_PKG_VERSION") {
                    " v" (pkg_version)
                }
            }
        }
        link rel="stylesheet" href="/resources/guest-default.css";
        body {
            @if let Some(pkg_name) = var("CARGO_PKG_NAME") {
                h1.guest-title {
                    (pkg_name)
                        @if let Some(pkg_ver) = var("CARGO_PKG_VERSION") {
                            span.guest-ver { " v" (pkg_ver) }
                        }
                }
            }

            @if let Some(pkg_repo) = var("CARGO_PKG_REPOSITORY") {
                h2.guest-repo {
                    "Repository: " a.guest-repo href={(pkg_repo)} {
                        (pkg_repo)
                    }
                }
            }
            (PreEscaped(desc_html))
            hr {}
            (PreEscaped(body))
        }
    }
    .into_string()
}
