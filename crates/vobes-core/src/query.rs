//! The shared query language.
//!
//! One parser, used by every face: the desktop search box, the CLI and
//! the MCP server. It used to live only in the desktop's TypeScript, so a
//! saved filter like `tag:rust` matched on the desktop and returned
//! nothing over MCP.
//!
//! Syntax (case-insensitive, whitespace-tolerant):
//!
//! ```text
//!   tag:<name>      vobe has a tag equal to <name>
//!   name:<x>        vobe name contains <x>
//!   lang:<x>        vobe language equals <x>
//!   fw:<x>          vobe framework equals <x>
//!   pm:<x>          vobe package manager equals <x>
//!   is:dirty        vobe.git.dirty === true
//!   is:clean        vobe.git.dirty === false
//!   is:behind       vobe.git.behind > 0
//!   is:ahead        vobe.git.ahead > 0
//!   is:unpushed     alias of is:ahead
//!   is:pinned       vobe.pinned
//!   is:attention    dirty, ahead or behind
//! ```
//!
//! Anything else is a free-text term, matched against name, path and
//! tags. Every term must match on its own, so `rust web` means "rust
//! **and** web".

use crate::vobe::Vobe;

/// One recognised predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// `tag:<name>` — the vobe carries this tag.
    Tag(String),
    /// `name:<x>` — the vobe name contains this text.
    Name(String),
    /// `lang:<x>` — the vobe's language equals this text.
    Language(String),
    /// `fw:<x>` — the vobe's framework equals this text.
    Framework(String),
    /// `pm:<x>` — the vobe's package manager equals this text.
    PackageManager(String),
    /// `is:dirty` — the working tree has uncommitted changes.
    Dirty,
    /// `is:clean` — the working tree is clean.
    Clean,
    /// `is:behind` — the branch is behind its upstream.
    Behind,
    /// `is:ahead` — the branch has commits to push.
    Ahead,
    /// `is:pinned` — the vobe is pinned.
    Pinned,
    /// `is:attention` — dirty, ahead or behind.
    Attention,
}

/// A parsed query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    /// Recognised predicates, in the order they were written.
    pub predicates: Vec<Predicate>,
    /// Free-text terms, lower-cased. Each must match on its own.
    pub terms: Vec<String>,
}

/// Find the vobe with this name among `all`.
///
/// Two projects in different roots can share a basename, so a name is not
/// unique. Picking one silently meant `vbs show app` and `vbs rm app`
/// acted on an arbitrary project. The caller gets the ambiguity instead
/// and can ask for a path.
pub fn find_by_name<'a>(all: &'a [Vobe], name: &str) -> crate::Result<Option<&'a Vobe>> {
    let mut hits = all.iter().filter(|v| v.name == name);
    let Some(first) = hits.next() else {
        return Ok(None);
    };
    if hits.next().is_some() {
        let paths: Vec<String> = all
            .iter()
            .filter(|v| v.name == name)
            .map(|v| v.path.display().to_string())
            .collect();
        return Err(crate::Error::invalid(format!(
            "`{name}` matches {} projects — use the path instead: {}",
            paths.len(),
            paths.join(", ")
        )));
    }
    Ok(Some(first))
}

impl Query {
    /// Parse a query string. An empty string matches everything.
    pub fn parse(q: &str) -> Self {
        let mut predicates = Vec::new();
        let mut terms = Vec::new();
        for token in q.split_whitespace() {
            let lower = token.to_lowercase();
            if let Some(p) = parse_predicate(&lower) {
                predicates.push(p);
            } else {
                terms.push(lower);
            }
        }
        Self { predicates, terms }
    }

    /// True when the vobe satisfies every predicate and every term.
    pub fn matches(&self, v: &Vobe) -> bool {
        for p in &self.predicates {
            if !predicate_matches(p, v) {
                return false;
            }
        }
        for term in &self.terms {
            if !term_matches(term, v) {
                return false;
            }
        }
        true
    }

    /// Keep only the vobes this query matches.
    pub fn filter<'a>(&self, vobes: impl IntoIterator<Item = &'a Vobe>) -> Vec<&'a Vobe> {
        vobes.into_iter().filter(|v| self.matches(v)).collect()
    }
}

/// Parse one token into a predicate. `None` means "not a predicate".
fn parse_predicate(token: &str) -> Option<Predicate> {
    if let Some(rest) = token.strip_prefix("tag:") {
        return (!rest.is_empty()).then(|| Predicate::Tag(rest.to_string()));
    }
    if let Some(rest) = token.strip_prefix("name:") {
        return (!rest.is_empty()).then(|| Predicate::Name(rest.to_string()));
    }
    if let Some(rest) = token.strip_prefix("lang:") {
        return (!rest.is_empty()).then(|| Predicate::Language(rest.to_string()));
    }
    if let Some(rest) = token.strip_prefix("fw:") {
        return (!rest.is_empty()).then(|| Predicate::Framework(rest.to_string()));
    }
    if let Some(rest) = token.strip_prefix("pm:") {
        return (!rest.is_empty()).then(|| Predicate::PackageManager(rest.to_string()));
    }
    match token {
        "is:dirty" => Some(Predicate::Dirty),
        "is:clean" => Some(Predicate::Clean),
        "is:behind" => Some(Predicate::Behind),
        "is:ahead" | "is:unpushed" => Some(Predicate::Ahead),
        "is:pinned" => Some(Predicate::Pinned),
        "is:attention" => Some(Predicate::Attention),
        _ => None,
    }
}

fn predicate_matches(p: &Predicate, v: &Vobe) -> bool {
    match p {
        Predicate::Tag(t) => v.tags.iter().any(|x| x.eq_ignore_ascii_case(t)),
        Predicate::Name(n) => v.name.to_lowercase().contains(n),
        Predicate::Language(l) => v
            .language
            .as_deref()
            .is_some_and(|x| x.eq_ignore_ascii_case(l)),
        Predicate::Framework(f) => v
            .framework
            .as_deref()
            .is_some_and(|x| x.eq_ignore_ascii_case(f)),
        Predicate::PackageManager(p) => v
            .package_manager
            .as_deref()
            .is_some_and(|x| x.eq_ignore_ascii_case(p)),
        Predicate::Dirty => v.git.as_ref().is_some_and(|g| g.dirty),
        Predicate::Clean => v.git.as_ref().is_some_and(|g| !g.dirty),
        Predicate::Behind => v.git.as_ref().is_some_and(|g| g.behind > 0),
        Predicate::Ahead => v.git.as_ref().is_some_and(|g| g.ahead > 0),
        Predicate::Pinned => v.pinned,
        Predicate::Attention => v
            .git
            .as_ref()
            .is_some_and(|g| g.dirty || g.ahead > 0 || g.behind > 0),
    }
}

fn term_matches(term: &str, v: &Vobe) -> bool {
    v.name.to_lowercase().contains(term)
        || v.path.to_string_lossy().to_lowercase().contains(term)
        || v.tags.iter().any(|t| t.to_lowercase().contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn vobe() -> Vobe {
        let mut v = Vobe::new("my-rust-web", PathBuf::from("/code/alpha/beta"));
        v.language = Some("Rust".into());
        v.framework = Some("Tauri".into());
        v.package_manager = Some("cargo".into());
        v.add_tag("backend");
        v
    }

    #[test]
    fn free_text_terms_match_independently() {
        let v = vobe();
        assert!(Query::parse("alpha beta").matches(&v));
        assert!(!Query::parse("alpha gamma").matches(&v));
    }

    #[test]
    fn predicates_are_recognised() {
        let v = vobe();
        assert!(Query::parse("tag:backend").matches(&v));
        assert!(!Query::parse("tag:other").matches(&v));
        assert!(Query::parse("lang:rust").matches(&v));
        assert!(Query::parse("fw:tauri").matches(&v));
        assert!(Query::parse("pm:cargo").matches(&v));
        assert!(Query::parse("name:my-rust").matches(&v));
    }

    #[test]
    fn is_predicates_read_git_state() {
        let mut v = vobe();
        assert!(!Query::parse("is:dirty").matches(&v));
        assert!(!Query::parse("is:attention").matches(&v));
        v.git = Some(crate::git::GitInfo {
            branch: "main".into(),
            dirty: true,
            ahead: 0,
            behind: 0,
            last_commit: None,
        });
        assert!(Query::parse("is:dirty").matches(&v));
        assert!(!Query::parse("is:clean").matches(&v));
        assert!(Query::parse("is:attention").matches(&v));
    }

    #[test]
    fn predicates_and_terms_combine() {
        let v = vobe();
        assert!(Query::parse("tag:backend alpha").matches(&v));
        assert!(!Query::parse("tag:backend gamma").matches(&v));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(Query::parse("").matches(&vobe()));
    }

    #[test]
    fn find_by_name_reports_ambiguity() {
        let a = vobe();
        let mut b = vobe();
        b.id = crate::error::VobeId::from_string("vobe_second");
        b.path = PathBuf::from("/other/alpha");
        let all = vec![a, b];
        let err = find_by_name(&all, "my-rust-web").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("2 projects"), "got {msg}");
        assert!(msg.contains("/other/alpha"), "got {msg}");
    }

    #[test]
    fn find_by_name_resolves_a_unique_name() {
        let all = vec![vobe()];
        let found = find_by_name(&all, "my-rust-web").unwrap();
        assert!(found.is_some());
        assert!(find_by_name(&all, "nope").unwrap().is_none());
    }
}
