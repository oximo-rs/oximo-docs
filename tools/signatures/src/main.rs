//! Collect API signatures from docs.rs for the oximo documentation site.
//!
//! The site links every API name to docs.rs. This tool walks those links, fetches
//! the corresponding docs.rs pages once per published documentation channel
//! (stable, each archived release, and the development channel), extracts the
//! rendered signature and a one-line summary for every referenced item, and
//! writes a single committed JSON index that `static/js/signatures.js` turns into
//! hover popovers.
//!
//! Run with `--help` for the available options.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use clap::Parser;
use regex::Regex;
use serde::Serialize;
use sha1::{Digest, Sha1};

const DOCS_RS: &str = "https://docs.rs/";
const USER_AGENT: &str = "oximo-docs-signature-collector (+https://oximo.dev)";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const SUMMARY_MAX_CHARS: usize = 240;
const SIGNATURE_MAX_CHARS: usize = 1200;

/// Rustdoc path prefixes that identify an anchored sub-item on a page.
const ANCHOR_PREFIXES: [&str; 6] = [
    "method.",
    "tymethod.",
    "associatedconstant.",
    "associatedtype.",
    "structfield.",
    "variant.",
];

// ---------------------------------------------------------------------------
// Regexes
// ---------------------------------------------------------------------------

fn url_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"https://docs\.rs/[^\s)"'<>\]`]+"#).unwrap())
}

fn tag_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"<[^>]+>").unwrap())
}

fn whitespace_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\s+").unwrap())
}

fn docblock_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?s)<div class=["']docblock["']>"#).unwrap())
}

fn paragraph_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?s)<p>(.*?)</p>").unwrap())
}

fn item_decl_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?s)<pre class="rust item-decl">(.*?)</pre>"#).unwrap())
}

fn code_header_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?s)<h[34] class="code-header">(.*?)</h[34]>"#).unwrap())
}

fn code_fence_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?s)```.*?```").unwrap())
}

fn inline_code_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"`[^`\n]+`").unwrap())
}

fn identifier_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap())
}

fn member_section_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r#"(?s)<section id="(method|tymethod|associatedconstant|associatedtype|variant)\.([A-Za-z0-9_]+)"[^>]*>(.*?)</section>"#,
        )
        .unwrap()
    })
}

fn struct_field_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"(?s)<span id="structfield\.([A-Za-z0-9_]+)"[^>]*>(.*?)</span>"#).unwrap()
    })
}

fn inline_code_re_any() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#"(?s)<code>(.*?)</code>"#).unwrap())
}

/// Rustdoc's collapsed-documentation "Read more" link, which trails inherited
/// one-line summaries.
fn read_more_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)\s*Read more\s*$").unwrap())
}

/// Links to items in docs.rs' `all.html` index, e.g. `prelude/macro.sum.html`.
fn all_items_link_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r#"href="([^"]*?(struct|enum|trait|union|type|fn|macro|constant|static)\.([A-Za-z0-9_]+)\.html)""#,
        )
        .unwrap()
    })
}

/// Blanket/std trait methods that rustdoc renders on every page.
const GENERIC_MEMBERS: [&str; 29] = [
    "clone", "clone_from", "clone_into", "clone_to_uninit", "to_owned", "to_smolstr",
    "to_string", "from", "into", "try_from", "try_into", "into_either",
    "into_either_with", "as_ref", "borrow", "borrow_mut", "by_ref", "deref", "deref_mut",
    "drop", "type_id", "fmt", "vzip", "instrument", "in_current_span", "with_subscriber",
    "with_current_subscriber", "init", "rand",
];

fn is_generic_member(name: &str) -> bool {
    GENERIC_MEMBERS.contains(&name)
}

// ---------------------------------------------------------------------------
// JSON model
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Index {
    generated: String,
    channels: BTreeMap<String, ChannelIndex>,
}

#[derive(Serialize)]
struct ChannelIndex {
    docs_rs_version: String,
    items: BTreeMap<String, Item>,
}

#[derive(Serialize)]
struct Item {
    signature: String,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

#[derive(Default)]
struct Stats {
    fetched: usize,
    cache_hits: usize,
    missing: usize,
    signatures_missing: usize,
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Collect docs.rs signatures into the oximo documentation index.
#[derive(Parser)]
#[command(name = "collect-signatures", version, long_about = None)]
struct Args {
    /// Site config.toml
    #[arg(long, value_name = "PATH", default_value_os_t = repo_root().join("config.toml"))]
    config: PathBuf,

    /// Output JSON
    #[arg(
        long,
        value_name = "PATH",
        default_value_os_t = repo_root().join("static").join("docs-signatures.json")
    )]
    output: PathBuf,

    /// Raw HTML cache
    #[arg(long, value_name = "PATH", default_value_os_t = repo_root().join(".signatures-cache"))]
    cache_dir: PathBuf,

    /// Comma-separated channels, e.g. --only 0.7.0,main
    #[arg(long, value_name = "CHANNELS", value_delimiter = ',')]
    only: Vec<String>,

    /// Ignore the local HTML cache and refetch every page
    #[arg(long)]
    refresh: bool,

    /// Pause between requests, in seconds
    #[arg(long, value_name = "SECONDS", default_value_t = 0.15)]
    delay: f64,
}

fn repo_root() -> PathBuf {
    // tools/signatures -> tools -> repository root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

// ---------------------------------------------------------------------------
// URL handling
// ---------------------------------------------------------------------------

struct DocsUrl {
    crate_name: String,
    rest: String,
    fragment: String,
}

/// Split a docs.rs URL into its crate, the channel-independent path after the
/// version segment, and the fragment. Returns `None` for URLs that do not
/// identify a versioned item page (bare crate roots, crate landing pages, and
/// non-docs.rs hosts).
fn split_docs_rs_url(url: &str) -> Option<DocsUrl> {
    let after_host = url.strip_prefix(DOCS_RS)?;
    let (path, fragment) = match after_host.split_once('#') {
        Some((path, fragment)) => (path, fragment),
        None => (after_host, ""),
    };
    let path = path.split_once('?').map(|(p, _)| p).unwrap_or(path);
    let segments: Vec<&str> = path.split('/').filter(|seg| !seg.is_empty()).collect();
    if segments.len() < 3 {
        return None;
    }
    Some(DocsUrl {
        crate_name: segments[0].to_string(),
        rest: segments[2..].join("/"),
        fragment: fragment.to_string(),
    })
}

/// Channel-independent lookup key shared with signatures.js.
fn item_key(crate_name: &str, rest: &str, fragment: &str) -> String {
    if fragment.is_empty() {
        format!("{crate_name}/{rest}")
    } else {
        format!("{crate_name}/{rest}#{fragment}")
    }
}

// ---------------------------------------------------------------------------
// Fetching
// ---------------------------------------------------------------------------

fn sha1_hex(value: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(value.as_bytes());
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Return the page HTML, or `None` when docs.rs reports the page missing.
fn fetch(
    url: &str,
    cache: &Path,
    refresh: bool,
    delay: Duration,
    stats: &mut Stats,
) -> Result<Option<String>> {
    let cached = cache.join(format!("{}.html", sha1_hex(url)));
    if !refresh {
        if let Ok(body) = fs::read_to_string(&cached) {
            stats.cache_hits += 1;
            return Ok(Some(body));
        }
    }

    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .http_status_as_error(true)
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build(),
    );
    match agent.get(url).header("User-Agent", USER_AGENT).call() {
        Ok(mut response) => {
            let body = response
                .body_mut()
                .read_to_string()
                .with_context(|| format!("reading response body for {url}"))?;
            stats.fetched += 1;
            if let Some(parent) = cached.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&cached, &body);
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            Ok(Some(body))
        }
        Err(ureq::Error::StatusCode(code)) if matches!(code, 400 | 403 | 404 | 410) => {
            stats.missing += 1;
            Ok(None)
        }
        Err(ureq::Error::StatusCode(code)) => bail!("docs.rs returned HTTP {code} for {url}"),
        Err(error) => {
            eprintln!("  ! network error for {url}: {error}");
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// HTML extraction
// ---------------------------------------------------------------------------

fn strip_tags(markup: &str) -> String {
    let without_tags = tag_re().replace_all(markup, "");
    html_escape::decode_html_entities(&without_tags).into_owned()
}

fn collapse_whitespace(text: &str) -> String {
    whitespace_re().replace_all(text, " ").trim().to_string()
}

/// Return the first paragraph of the first docblock that follows a section in
/// `region`.
fn first_paragraph(region: &str) -> Option<String> {
    let limit = [region.find("</details>"), region.find("<section id=")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(region.len());
    let region = &region[..limit];

    let docblock = docblock_re().find(region)?;
    let rest = &region[docblock.end()..];
    let raw = match paragraph_re().captures(rest) {
        Some(paragraph) => paragraph[1].to_string(),
        // Trait-impl summaries are bare text (no wrapping <p>).
        None => {
            let end = rest.find("</div>").unwrap_or(rest.len());
            rest[..end].to_string()
        }
    };
    let text = collapse_whitespace(&strip_tags(&mark_inline_code(&raw)));
    let text = strip_read_more(&text);
    if text.is_empty() {
        return None;
    }
    Some(truncate_words(&text, SUMMARY_MAX_CHARS))
}

/// Rewrite inline `<code>...</code>` spans as backtick-delimited text so the
/// popover can render them as code.
fn mark_inline_code(raw: &str) -> String {
    inline_code_re_any()
        .replace_all(raw, |caps: &regex::Captures| {
            format!("`{}`", tag_re().replace_all(caps[1].trim(), ""))
        })
        .into_owned()
}

/// Drop rustdoc's trailing "Read more" toggle text from a summary.
fn strip_read_more(text: &str) -> String {
    read_more_re().replace(text, "").trim_end().to_string()
}

fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut = text
        .char_indices()
        .nth(max)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    match text[..cut].rfind(' ') {
        Some(space) if space > 0 => format!("{}...", &text[..space]),
        _ => format!("{text}..."),
    }
}

/// Preserve newlines (multi-line declarations such as structs) but tidy the
/// trailing whitespace rustdoc leaves behind, and cap very long declarations.
fn clean_signature(markup: &str) -> String {
    let text = strip_tags(markup);
    let joined = text
        .split('\n')
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = joined.trim_matches('\n').trim();
    cap_signature(trimmed, SIGNATURE_MAX_CHARS)
}

fn cap_signature(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut cut = text
        .char_indices()
        .nth(max)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    if let Some(newline) = text[..cut].rfind('\n') {
        if newline > 0 {
            cut = newline;
        }
    }
    format!("{}\n…", text[..cut].trim_end())
}

/// Extract `(signature, summary)` for an item or anchored sub-item.
fn extract_item(page: &str, fragment: &str) -> (Option<String>, Option<String>) {
    if !fragment.is_empty() && ANCHOR_PREFIXES.iter().any(|prefix| fragment.starts_with(prefix)) {
        let pattern = format!(
            r#"(?s)<section id="{}"[^>]*>(.*?)</section>"#,
            regex::escape(fragment)
        );
        let section = match Regex::new(&pattern).unwrap().captures(page) {
            Some(section) => section,
            None => return (None, None),
        };
        let header = match code_header_re().captures(&section[1]) {
            Some(header) => header,
            None => return (None, None),
        };
        let end = section.get(0).unwrap().end();
        let signature = clean_signature(&header[1]);
        if signature.is_empty() {
            return (None, None);
        }
        let summary = first_paragraph(&page[end..]);
        return (Some(signature), summary);
    }

    let declaration = match item_decl_re().captures(page) {
        Some(declaration) => declaration,
        None => return (None, None),
    };
    let signature = clean_signature(&declaration[1]);
    if signature.is_empty() {
        return (None, None);
    }
    let end = declaration.get(0).unwrap().end();
    let summary = first_paragraph(&page[end..]);
    (Some(signature), summary)
}

/// Extract every documented member (methods, associated items, variants) whose
/// name appears in the documentation's code. These make hovering over calls such
/// as `result.objective()` work, not just the items that are linked.
fn extract_members(
    page: &str,
    crate_name: &str,
    rest: &str,
    source: &str,
    candidates: &BTreeSet<String>,
) -> Vec<(String, Item)> {
    let mut members = Vec::new();
    for section in member_section_re().captures_iter(page) {
        let kind = &section[1];
        let name = &section[2];
        if !candidates.contains(name) || is_generic_member(name) {
            continue;
        }
        let Some(header) = code_header_re().captures(&section[3]) else {
            continue;
        };
        let signature = clean_signature(&header[1]);
        if signature.is_empty() {
            continue;
        }
        let summary = first_paragraph(&page[section.get(0).unwrap().end()..]);
        let key = format!("{crate_name}/{rest}#{kind}.{name}");
        members.push((
            key,
            Item {
                signature,
                source: source.to_string(),
                summary,
            },
        ));
    }

    for field in struct_field_re().captures_iter(page) {
        let name = &field[1];
        if !candidates.contains(name) || is_generic_member(name) {
            continue;
        }
        let Some(code) = inline_code_re_any().captures(&field[2]) else {
            continue;
        };
        let declaration = clean_signature(&code[1]);
        if declaration.is_empty() {
            continue;
        }
        let key = format!("{crate_name}/{rest}#structfield.{name}");
        members.push((
            key,
            Item {
                signature: format!("pub {declaration}"),
                source: source.to_string(),
                summary: None,
            },
        ));
    }

    members
}

// ---------------------------------------------------------------------------
// Configuration and discovery
// ---------------------------------------------------------------------------

/// Return the ordered `(channel, docs.rs version)` plan to collect.
fn channel_plan(config_path: &Path) -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string(config_path)
        .with_context(|| format!("reading {}", config_path.display()))?;
    let config: toml::Value = toml::from_str(&text).context("parsing config.toml")?;
    let extra = config.get("extra").context("missing [extra] in config.toml")?;

    let mut plan: Vec<(String, String)> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    if let Some(list) = extra
        .get("versions")
        .and_then(|versions| versions.get("list"))
        .and_then(toml::Value::as_array)
    {
        for entry in list {
            if let Some(version) = entry.get("version").and_then(toml::Value::as_str) {
                if seen.insert(version.to_string()) {
                    plan.push((version.to_string(), version.to_string()));
                }
            }
        }
    }
    if let Some(stable) = extra.get("stable_version").and_then(toml::Value::as_str) {
        if seen.insert(stable.to_string()) {
            plan.push((stable.to_string(), stable.to_string()));
        }
    }
    // The development channel follows unreleased changes; docs.rs can only offer
    // the newest published release for it.
    plan.push(("main".to_string(), "latest".to_string()));
    Ok(plan)
}

fn discover_referenced_urls(roots: &[PathBuf]) -> BTreeSet<String> {
    let mut urls = BTreeSet::new();
    let regex = url_re();
    for root in roots {
        for file in markdown_files(root) {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            for found in regex.find_iter(&text) {
                let url = found.as_str().trim_end_matches(['.', ',', ';', ':']);
                urls.insert(url.to_string());
            }
        }
    }
    urls
}

/// Identifiers that appear inside the documentation's code (fenced blocks and
/// inline spans). Members of referenced types are only indexed when one of these
/// names shows up, which keeps the index small and relevant.
fn discover_candidate_names(roots: &[PathBuf]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for root in roots {
        for file in markdown_files(root) {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let mut code = String::new();
            for fenced in code_fence_re().find_iter(&text) {
                code.push_str(fenced.as_str());
            }
            for inline in inline_code_re().find_iter(&text) {
                code.push_str(inline.as_str());
            }
            for identifier in identifier_re().find_iter(&code) {
                names.insert(identifier.as_str().to_string());
            }
        }
    }
    names
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown(root, &mut files);
    files
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Collection
// ---------------------------------------------------------------------------

/// Macros and free functions that documentation code uses but never links to.
/// They are discovered from the crate's `all.html` index and filtered to names
/// that actually appear in the docs.
fn discover_crate_items(
    crate_name: &str,
    docs_version: &str,
    candidates: &BTreeSet<String>,
    cache: &Path,
    refresh: bool,
    delay: Duration,
    stats: &mut Stats,
) -> Result<BTreeSet<String>> {
    let mut items = BTreeSet::new();
    let mut version_candidates = vec![docs_version.to_string()];
    if docs_version != "latest" {
        version_candidates.push("latest".to_string());
    }

    for candidate in &version_candidates {
        let module = crate_name.replace('-', "_");
        let url = format!("{DOCS_RS}{crate_name}/{candidate}/{module}/all.html");
        let Some(page) = fetch(&url, cache, refresh, delay, stats)? else {
            continue;
        };
        for link in all_items_link_re().captures_iter(&page) {
            let kind = &link[2];
            let name = &link[3];
            if kind != "macro" && kind != "fn" {
                continue;
            }
            if !candidates.contains(name) {
                continue;
            }
            let href = link[1].trim_start_matches("./");
            if href.starts_with("http") {
                continue;
            }
            items.insert(format!("{crate_name}/{module}/{href}"));
        }
        break;
    }

    Ok(items)
}

fn collect_channel(
    docs_version: &str,
    urls: &BTreeSet<String>,
    candidates: &BTreeSet<String>,
    cache: &Path,
    refresh: bool,
    delay: Duration,
    stats: &mut Stats,
) -> Result<BTreeMap<String, Item>> {
    let mut by_page: BTreeMap<String, Vec<DocsUrl>> = BTreeMap::new();
    for url in urls {
        if let Some(parsed) = split_docs_rs_url(url) {
            let fingerprint = format!("{}/{}", parsed.crate_name, parsed.rest);
            by_page.entry(fingerprint).or_default().push(parsed);
        }
    }

    // Add macros and free functions the docs use in code but never link to.
    let crates: BTreeSet<String> = by_page
        .keys()
        .filter_map(|fingerprint| fingerprint.split_once('/').map(|(name, _)| name.to_string()))
        .collect();
    for crate_name in &crates {
        for fingerprint in
            discover_crate_items(crate_name, docs_version, candidates, cache, refresh, delay, stats)?
        {
            by_page.entry(fingerprint).or_default();
        }
    }

    let mut index: BTreeMap<String, Item> = BTreeMap::new();
    for (fingerprint, references) in &by_page {
        let (crate_name, rest) = fingerprint
            .split_once('/')
            .expect("fingerprint always contains crate and rest");

        let mut version_candidates = vec![docs_version.to_string()];
        if docs_version != "latest" {
            version_candidates.push("latest".to_string());
        }

        let mut page = None;
        let mut source = String::new();
        for candidate in &version_candidates {
            let url = format!("{DOCS_RS}{crate_name}/{candidate}/{rest}");
            if let Some(body) = fetch(&url, cache, refresh, delay, stats)? {
                page = Some(body);
                source = candidate.clone();
                break;
            }
        }
        let Some(page) = page else {
            continue;
        };

        if references.is_empty() {
            // Discovered item (e.g. a macro): index its top-level declaration.
            if let (Some(signature), summary) = extract_item(&page, "") {
                index.insert(
                    item_key(crate_name, rest, ""),
                    Item {
                        signature,
                        source: source.clone(),
                        summary,
                    },
                );
            } else {
                stats.signatures_missing += 1;
            }
        } else {
            for reference in references {
                match extract_item(&page, &reference.fragment) {
                    (Some(signature), summary) => {
                        let key = item_key(crate_name, rest, &reference.fragment);
                        index.insert(
                            key,
                            Item {
                                signature,
                                source: source.clone(),
                                summary,
                            },
                        );
                    }
                    _ => stats.signatures_missing += 1,
                }
            }
        }

        // Index the page's other documented members (methods, variants, …) so
        // code references such as `result.objective()` get a tooltip too.
        for (key, item) in extract_members(&page, crate_name, rest, &source, candidates) {
            index.entry(key).or_insert(item);
        }
    }

    Ok(index)
}

// ---------------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------------

fn utc_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let (hour, minute, second) = (
        seconds_of_day / 3600,
        (seconds_of_day % 3600) / 60,
        seconds_of_day % 60,
    );
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let args = Args::parse();
    let only: Vec<String> = args
        .only
        .iter()
        .map(|channel| channel.trim().to_string())
        .filter(|channel| !channel.is_empty())
        .collect();
    let delay = Duration::from_secs_f64(args.delay.max(0.0));
    let root = repo_root();
    let roots = [
        root.join("content"),
        root.join("dev"),
        root.join("versions"),
    ];

    let urls = discover_referenced_urls(&roots);
    println!("Referenced docs.rs URLs: {}", urls.len());
    let candidates = discover_candidate_names(&roots);
    println!("Candidate code identifiers: {}", candidates.len());

    let mut plan = channel_plan(&args.config)?;
    if !only.is_empty() {
        plan.retain(|(channel, _)| only.contains(channel));
    }
    if plan.is_empty() {
        bail!("no channels selected");
    }

    let mut channels = BTreeMap::new();
    for (channel, docs_version) in &plan {
        let mut stats = Stats::default();
        println!("Collecting channel {channel} (docs.rs {docs_version}) ...");
        let items = collect_channel(
            docs_version,
            &urls,
            &candidates,
            &args.cache_dir,
            args.refresh,
            delay,
            &mut stats,
        )?;
        println!(
            "  {} signatures ({} fetched, {} cached, {} not found, {} unparsed)",
            items.len(),
            stats.fetched,
            stats.cache_hits,
            stats.missing,
            stats.signatures_missing,
        );
        if !items.is_empty() {
            channels.insert(
                channel.clone(),
                ChannelIndex {
                    docs_rs_version: docs_version.clone(),
                    items,
                },
            );
        }
    }

    if channels.is_empty() {
        bail!("no signatures collected; leaving the existing index untouched");
    }

    let index = Index {
        generated: utc_timestamp(),
        channels,
    };
    let encoded = serde_json::to_string_pretty(&index).context("serializing signature index")?;
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&args.output, format!("{encoded}\n"))
        .with_context(|| format!("writing {}", args.output.display()))?;
    println!("Wrote {}", args.output.display());
    Ok(())
}
