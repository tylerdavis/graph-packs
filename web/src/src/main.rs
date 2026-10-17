mod dom;
mod page;
mod search;

use serde_json::json;
use std::process::ExitCode;
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";
const DEFAULT_NUM: usize = 10;
const MAX_NUM: usize = 20;
const DEFAULT_MAX_CHARS: usize = 40_000;
const MAX_BODY_BYTES: u64 = 20 * 1024 * 1024;

const USAGE: &str = "usage:\n  web search <query> [--num N]\n  web fetch <url> [--max-chars N]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("web: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<serde_json::Value, String> {
    let (command, rest) = args.split_first().ok_or(USAGE)?;
    match command.as_str() {
        "search" => {
            let (words, num) = split_flag(rest, "--num")?;
            let query = words.join(" ");
            if query.trim().is_empty() {
                return Err(format!("search needs a query\n{USAGE}"));
            }
            let num = num.unwrap_or(DEFAULT_NUM).clamp(1, MAX_NUM);
            search(query.trim(), num)
        }
        "fetch" => {
            let (words, max_chars) = split_flag(rest, "--max-chars")?;
            let [url] = words.as_slice() else {
                return Err(format!("fetch needs exactly one URL\n{USAGE}"));
            };
            fetch(url, max_chars.unwrap_or(DEFAULT_MAX_CHARS).max(1))
        }
        "-h" | "--help" | "help" => Err(USAGE.to_string()),
        other => Err(format!("unknown command '{other}'\n{USAGE}")),
    }
}

fn split_flag(args: &[String], flag: &str) -> Result<(Vec<String>, Option<usize>), String> {
    let mut words = Vec::new();
    let mut value = None;
    let mut iter = args.iter();
    let prefix = format!("{flag}=");
    while let Some(arg) = iter.next() {
        let raw = if arg == flag {
            Some(iter.next().ok_or(format!("{flag} needs a number"))?.clone())
        } else {
            arg.strip_prefix(&prefix).map(str::to_string)
        };
        match raw {
            Some(raw) if raw.trim().is_empty() => {}
            Some(raw) => {
                value = Some(
                    raw.trim()
                        .parse::<usize>()
                        .map_err(|_| format!("{flag} needs a whole number, got '{raw}'"))?,
                )
            }
            None => words.push(arg.clone()),
        }
    }
    Ok((words, value))
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("setting up HTTP: {e}"))
}

fn send(
    request: reqwest::blocking::RequestBuilder,
    what: &str,
) -> Result<reqwest::blocking::Response, String> {
    let response = request
        .send()
        .map_err(|e| format!("{what}: {}", describe(&e)))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("{what} answered HTTP {}", status.as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BODY_BYTES)
    {
        return Err(format!("{what} sent more than {MAX_BODY_BYTES} bytes"));
    }
    Ok(response)
}

fn describe(error: &reqwest::Error) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    message
}

fn read_body(response: reqwest::blocking::Response) -> Result<String, String> {
    response
        .text()
        .map_err(|e| format!("reading the response: {}", describe(&e)))
}

fn search(query: &str, num: usize) -> Result<serde_json::Value, String> {
    let backend = search::Backend::from_env(
        std::env::var("SEARXNG_URL").ok(),
        std::env::var("BRAVE_API_KEY").ok(),
    );
    let client = client()?;
    let hits = match &backend {
        search::Backend::Searxng(base) => {
            let endpoint = format!("{}/search", base.trim_end_matches('/'));
            let request = client
                .get(&endpoint)
                .query(&[("q", query), ("format", "json")])
                .header("Accept", "application/json");
            search::parse_searxng(&read_body(send(request, &format!("SearXNG at {base}"))?)?)?
        }
        search::Backend::BraveApi(key) => {
            let request = client
                .get("https://api.search.brave.com/res/v1/web/search")
                .query(&[("q", query), ("count", &num.to_string())])
                .header("Accept", "application/json")
                .header("X-Subscription-Token", key);
            search::parse_brave_api(&read_body(send(
                request,
                "the Brave Search API with BRAVE_API_KEY",
            )?)?)?
        }
        search::Backend::Brave => {
            let request = client
                .get("https://search.brave.com/search")
                .query(&[("q", query), ("source", "web")])
                .header("Accept", "text/html,application/xhtml+xml")
                .header("Accept-Language", "en-US,en;q=0.9");
            let response = send(request, "search.brave.com").map_err(|e| {
                format!(
                    "{e}; set SEARXNG_URL or BRAVE_API_KEY to search through a configured backend"
                )
            })?;
            search::parse_brave_page(&read_body(response)?)
        }
    };
    let results: Vec<search::Hit> = hits.into_iter().take(num).collect();
    Ok(json!({
        "query": query,
        "backend": backend.name(),
        "results": results,
    }))
}

fn fetch(raw_url: &str, max_chars: usize) -> Result<serde_json::Value, String> {
    let url = url::Url::parse(raw_url).map_err(|e| format!("'{raw_url}' is not a URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!(
            "only http and https URLs can be fetched, got '{raw_url}'"
        ));
    }
    let request = client()?
        .get(url.as_str())
        .header(
            "Accept",
            "text/html,application/xhtml+xml,text/markdown,text/plain;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9");
    let response = send(request, url.as_str())?;
    let final_url = response.url().clone();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    if is_binary(&content_type) {
        return Err(format!(
            "{final_url} is {content_type}, which can't be read as text"
        ));
    }
    let body = read_body(response)?;
    let looks_html = content_type.contains("html")
        || (content_type.is_empty() && body.trim_start().starts_with('<'));
    let (title, markdown) = if looks_html {
        let page = page::convert(&body, Some(&final_url));
        (page.title, page.markdown)
    } else {
        (None, body.trim().to_string())
    };
    let (markdown, truncated) = page::truncate(&markdown, max_chars);
    Ok(json!({
        "url": final_url.as_str(),
        "title": title,
        "markdown": markdown,
        "truncated": truncated,
    }))
}

fn is_binary(content_type: &str) -> bool {
    let kind = content_type.split(';').next().unwrap_or("").trim();
    if kind.is_empty() || kind.starts_with("text/") {
        return false;
    }
    let textual = [
        "json",
        "xml",
        "javascript",
        "yaml",
        "toml",
        "x-sh",
        "graphql",
    ];
    !textual.iter().any(|marker| kind.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn flags_parse_in_either_spelling_and_blank_values_mean_the_default() {
        assert_eq!(
            split_flag(&strings(&["a", "b", "--num", "5"]), "--num").unwrap(),
            (strings(&["a", "b"]), Some(5))
        );
        assert_eq!(
            split_flag(&strings(&["--num=3", "q"]), "--num").unwrap(),
            (strings(&["q"]), Some(3))
        );
        assert_eq!(
            split_flag(&strings(&["q", "--num", ""]), "--num").unwrap(),
            (strings(&["q"]), None)
        );
        assert!(split_flag(&strings(&["q", "--num", "x"]), "--num").is_err());
    }

    #[test]
    fn bad_commands_and_urls_are_errors() {
        assert!(run(&[]).is_err());
        assert!(run(&strings(&["nope"]))
            .unwrap_err()
            .contains("unknown command"));
        assert!(run(&strings(&["fetch", "ftp://example.com"]))
            .unwrap_err()
            .contains("http"));
        assert!(run(&strings(&["fetch", "not a url"]))
            .unwrap_err()
            .contains("not a URL"));
        assert!(run(&strings(&["search", " "]))
            .unwrap_err()
            .contains("needs a query"));
    }

    #[test]
    fn text_formats_are_read_and_binary_ones_refused() {
        for text in [
            "text/html; charset=utf-8",
            "application/json",
            "application/xml",
            "",
        ] {
            assert!(!is_binary(text), "{text}");
        }
        for binary in ["application/pdf", "image/png", "application/octet-stream"] {
            assert!(is_binary(binary), "{binary}");
        }
    }
}
