//! `REASSOC_TRACE=<file>`: an audit log of what the macros did.
//!
//! Set in the environment of a build, every function the macros enter and
//! every `alg!` invocation appends one line to the file:
//!
//! ```text
//! <file>:<line>\t<kind>\t<name>\t<operators rewritten>
//! ```
//!
//! `kind` is `fn` (a function body entered by `#[algebraic]`, directly or as a
//! member of an annotated container), `const fn` (met in an algebraic scope and
//! left as written, so always 0), or `alg` (one `alg!`, named `-`). Nothing in
//! the generated code changes; the log is for tooling that asks "which
//! functions did the macro reach, and where did it find nothing to rewrite".
//! `scripts/adopt/` in the repository uses it. Lines are appended, so a build
//! that expands a crate more than once (check, test, doc) repeats them;
//! readers de-duplicate on the location.

use std::io::Write;
use std::sync::OnceLock;

fn path() -> Option<&'static str> {
    static PATH: OnceLock<Option<String>> = OnceLock::new();
    PATH.get_or_init(|| {
        std::env::var("REASSOC_TRACE")
            .ok()
            .filter(|p| !p.is_empty())
    })
    .as_deref()
}

/// One line; `span` locates the item (its name's span, or the macro's).
pub fn record(kind: &str, span: proc_macro2::Span, name: &str, ops: usize) {
    let Some(path) = path() else { return };
    // Only a real proc-macro context can resolve a span to a file.
    let (file, line) = if proc_macro::is_available() {
        let s = span.unwrap();
        (s.file(), s.line())
    } else {
        (String::from("?"), 0)
    };
    // One `write` per line: several rustc processes (a library and its test
    // targets) append to the same file, and `writeln!` on a `File` would
    // issue one syscall per formatted piece and tear lines.
    let line = format!("{file}:{line}\t{kind}\t{name}\t{ops}\n");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}
