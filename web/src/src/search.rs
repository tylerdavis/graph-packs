use crate::dom;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    Searxng(String),
    BraveApi(String),
    Brave,
}

impl Backend {
    pub fn from_env(searxng: Option<String>, brave_key: Option<String>) -> Self {
        let set = |value: Option<String>| {
            value
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        if let Some(url) = set(searxng) {
            return Backend::Searxng(url);
        }
        if let Some(key) = set(brave_key) {
            return Backend::BraveApi(key);
        }
        Backend::Brave
    }

    pub fn name(&self) -> &'static str {
        match self {
            Backend::Searxng(_) => "searxng",
            Backend::BraveApi(_) => "brave_api",
            Backend::Brave => "brave",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hit {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Deserialize)]
struct SearxngResponse {
    #[serde(default)]
    results: Vec<SearxngResult>,
}

#[derive(Deserialize)]
struct SearxngResult {
    #[serde(default)]
    title: String,
    url: String,
    #[serde(default)]
    content: Option<String>,
}

pub fn parse_searxng(body: &str) -> Result<Vec<Hit>, String> {
    let response: SearxngResponse = serde_json::from_str(body)
        .map_err(|e| format!("SearXNG did not return JSON search results ({e}); is format=json enabled in its settings.yml?"))?;
    Ok(response
        .results
        .into_iter()
        .map(|result| Hit {
            title: clean(&result.title),
            url: result.url,
            snippet: result.content.as_deref().map(clean).unwrap_or_default(),
        })
        .collect())
}

pub fn parse_brave_api(body: &str) -> Result<Vec<Hit>, String> {
    let response: Value = serde_json::from_str(body)
        .map_err(|e| format!("the Brave Search API did not return JSON ({e})"))?;
    let results = response
        .pointer("/web/results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(results
        .iter()
        .filter_map(|result| {
            let url = result.get("url")?.as_str()?.to_string();
            let field = |key: &str| {
                result
                    .get(key)
                    .and_then(Value::as_str)
                    .map(clean)
                    .unwrap_or_default()
            };
            Some(Hit {
                title: field("title"),
                url,
                snippet: field("description"),
            })
        })
        .collect())
}

pub fn parse_brave_page(html: &str) -> Vec<Hit> {
    let document = dom::parse(html);
    let mut results = Vec::new();
    dom::find_all(
        &document,
        &|node| dom::attr(node, "data-type").as_deref() == Some("web"),
        &mut results,
    );
    results
        .iter()
        .filter_map(|result| {
            let link = dom::find(result, &|node| {
                dom::tag(node).as_deref() == Some("a")
                    && dom::attr(node, "href").is_some_and(|href| href.starts_with("http"))
            })?;
            let url = dom::attr(&link, "href")?;
            let title = dom::find(&link, &|node| dom::has_class(node, "title"))
                .map(|node| dom::attr(&node, "title").unwrap_or_else(|| dom::text(&node)))
                .unwrap_or_else(|| dom::text(&link));
            let snippet = ["generic-snippet", "content", "description"]
                .iter()
                .find_map(|class| {
                    dom::find(result, &|node| dom::has_class(node, class))
                        .map(|node| dom::text(&node))
                        .filter(|text| !text.is_empty())
                })
                .unwrap_or_default();
            Some(Hit {
                title: title.trim().to_string(),
                url,
                snippet,
            })
        })
        .collect()
}

fn clean(fragment: &str) -> String {
    if !fragment.contains('<') && !fragment.contains('&') {
        return fragment.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    dom::text(&dom::parse(fragment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searxng_wins_over_a_brave_key_and_blank_values_are_unset() {
        assert_eq!(
            Backend::from_env(Some("https://sx.example".into()), Some("key".into())),
            Backend::Searxng("https://sx.example".into())
        );
        assert_eq!(
            Backend::from_env(Some("  ".into()), Some("key".into())),
            Backend::BraveApi("key".into())
        );
        assert_eq!(Backend::from_env(None, Some(String::new())), Backend::Brave);
    }

    #[test]
    fn parses_searxng_json_results() {
        let hits = parse_searxng(include_str!("../fixtures/searxng.json")).unwrap();
        assert_eq!(hits.len(), 3);
        assert_eq!(
            hits[0],
            Hit {
                title: "Create a customer | Stripe API Reference".into(),
                url: "https://docs.stripe.com/api/customers/create".into(),
                snippet: "Creates a new customer object.".into(),
            }
        );
        assert_eq!(hits[1].snippet, "");
    }

    #[test]
    fn a_searxng_html_page_is_an_error_that_names_the_format_setting() {
        let error = parse_searxng("<html></html>").unwrap_err();
        assert!(error.contains("format=json"), "{error}");
    }

    #[test]
    fn parses_brave_api_results_and_strips_highlighting() {
        let hits = parse_brave_api(include_str!("../fixtures/brave_api.json")).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].title, "Create a customer | Stripe API Reference");
        assert_eq!(hits[0].url, "https://docs.stripe.com/api/customers/create");
        assert_eq!(
            hits[0].snippet,
            "Creates a new customer object. Returns the Customer object after successful customer creation."
        );
        assert_eq!(
            hits[1].snippet,
            "This object represents a customer of your business & lets you track payments."
        );
    }

    #[test]
    fn parses_the_brave_results_page() {
        let hits = parse_brave_page(include_str!("../fixtures/brave.html"));
        let urls: Vec<&str> = hits.iter().map(|hit| hit.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://docs.stripe.com/api/customers/create",
                "https://docs.stripe.com/api/customers",
                "https://stackoverflow.com/questions/51144576/stripe-create-retrieve-customer-in-one-call",
                "https://egghead.io/lessons/supabase-create-a-stripe-customer-with-next-js-api-routes",
            ]
        );
        assert_eq!(hits[0].title, "Create a customer | Stripe API Reference");
        assert!(
            hits[0]
                .snippet
                .starts_with("curl https://api.stripe.com/v1/customers"),
            "{}",
            hits[0].snippet
        );
        assert!(
            hits[3].snippet.contains("use our private API key"),
            "{}",
            hits[3].snippet
        );
        assert!(hits.iter().all(|hit| !hit.title.is_empty()));
    }
}
