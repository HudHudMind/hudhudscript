//! Statement/function coverage collected by instrumenting the AST before
//! compilation: a `__hudunit_mark("<id>")` call is inserted before every
//! statement in every block, and each id is mapped back to the original
//! source span. At runtime the prelude's `__hudunit_mark` appends ids to the
//! global `__hudunit_seen` string, which the runner drains after each test
//! and feeds into [`Marks`]. Nothing on disk is ever modified.
//!
//! Imported `src/` modules are covered too: the module resolver instruments
//! module function bodies before handing the formatted source to the VM's
//! module loader. Modules execute their top level in a sub-VM, but exported
//! function bodies are merged into the host bytecode and run on the HOST VM
//! (verified behavior), so their marks reach the host's `__hudunit_seen`
//! during tests. Module top-level statements are therefore not instrumented
//! (the helper is not in scope there); only function bodies are.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use hudhudscript_ast::{Expr, Literal, Span, Stmt};
use hudhudscript_errors::{Error, ModuleContent, ModuleResolver};

/// Runtime hit-counts keyed by mark id.
#[derive(Clone, Default)]
pub struct Marks(Arc<Mutex<HashMap<usize, u64>>>);

impl Marks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit(&self, id: usize) {
        *self.0.lock().unwrap().entry(id).or_insert(0) += 1;
    }

    pub fn get(&self, id: usize) -> u64 {
        self.0.lock().unwrap().get(&id).copied().unwrap_or(0)
    }

    /// Record one execution of every id in a `__hudunit_seen` payload
    /// (`"3,7,12,"`). Malformed segments are skipped, not fatal.
    pub fn absorb_seen(&self, seen: &str) {
        for part in seen.split(',') {
            if part.is_empty() {
                continue;
            }
            if let Ok(id) = part.parse::<usize>() {
                self.hit(id);
            }
        }
    }
}

/// Static information about one instrumented statement.
#[derive(Debug, Clone)]
pub struct MarkInfo {
    /// The mark id (filled in by `MarkMap::alloc`).
    pub id: usize,
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    /// Enclosing function (entry marks carry the function name).
    pub function: Option<String>,
}

/// Mark id → static info; also the id allocator.
#[derive(Default)]
pub struct MarkMap {
    next_id: usize,
    entries: HashMap<usize, MarkInfo>,
}

impl MarkMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&mut self, mut info: MarkInfo) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        info.id = id;
        self.entries.insert(id, info);
        id
    }

    pub fn info(&self, id: usize) -> Option<&MarkInfo> {
        self.entries.get(&id)
    }

    pub fn all(&self) -> &HashMap<usize, MarkInfo> {
        &self.entries
    }
}

/// Instrument a statement list in place: recursion-safe (prelude helpers are
/// skipped) and span-preserving (marks reference the original file's lines).
pub fn instrument(stmts: &mut Vec<Stmt>, file: &Path, map: &mut MarkMap) {
    instrument_list(stmts, file, map, None);
}

/// Instrument an IMPORTED module: only function bodies get marks. The module
/// top level runs in a sub-VM where the host's `__hudunit_mark` helper does
/// not exist, so a top-level mark would crash the import; exported function
/// bodies, by contrast, run on the host VM where the helper is in scope.
pub fn instrument_module(stmts: &mut [Stmt], file: &Path, map: &mut MarkMap) {
    for stmt in stmts.iter_mut() {
        walk_stmt(stmt, file, map, None);
    }
}

/// Prelude functions that must never be instrumented (their mark call would
/// recurse forever, and their spans do not exist in the user's file).
fn is_prelude_helper(name: &str) -> bool {
    name.starts_with("__hudunit_") || name.starts_with("assert")
}

fn instrument_list(list: &mut Vec<Stmt>, file: &Path, map: &mut MarkMap, current_fn: Option<&str>) {
    let mut out: Vec<Stmt> = Vec::with_capacity(list.len() * 2);
    for mut stmt in list.drain(..) {
        walk_stmt(&mut stmt, file, map, current_fn);
        let span = stmt.span();
        let is_function = matches!(stmt, Stmt::Function { .. });
        if !is_function {
            let id = map.alloc(MarkInfo {
                id: 0,
                file: file.to_path_buf(),
                start_line: span.start.line,
                end_line: span.end.line,
                function: current_fn.map(|s| s.to_string()),
            });
            out.push(mark_stmt(id, span));
        }
        out.push(stmt);
    }
    *list = out;
}

fn walk_stmt(stmt: &mut Stmt, file: &Path, map: &mut MarkMap, current_fn: Option<&str>) {
    match stmt {
        Stmt::Function { name, body, span, .. } => {
            if is_prelude_helper(name) {
                return;
            }
            let entry_id = map.alloc(MarkInfo {
                id: 0,
                file: file.to_path_buf(),
                start_line: span.start.line,
                end_line: span.start.line,
                function: Some(name.clone()),
            });
            let mut prologue = vec![mark_stmt(entry_id, *span)];
            prologue.append(body);
            *body = prologue;
            instrument_list(body, file, map, Some(name));
        }
        Stmt::If { then_branch, else_branch, .. } => {
            walk_boxed(then_branch, file, map, current_fn);
            if let Some(else_branch) = else_branch {
                walk_boxed(else_branch, file, map, current_fn);
            }
        }
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForRange { body, .. } => {
            walk_boxed(body, file, map, current_fn);
        }
        Stmt::ForCStyle { init, update, body, .. } => {
            if let Some(init) = init {
                walk_boxed(init, file, map, current_fn);
            }
            if let Some(update) = update {
                walk_boxed(update, file, map, current_fn);
            }
            walk_boxed(body, file, map, current_fn);
        }
        Stmt::Block { statements, .. } => instrument_list(statements, file, map, current_fn),
        Stmt::Switch { cases, default, .. } => {
            for case in cases {
                instrument_list(&mut case.body, file, map, current_fn);
            }
            if let Some(default) = default {
                instrument_list(default, file, map, current_fn);
            }
        }
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            walk_boxed(try_block, file, map, current_fn);
            if let Some(catch_clause) = catch_clause {
                walk_boxed(&mut catch_clause.body, file, map, current_fn);
            }
            if let Some(finally_block) = finally_block {
                walk_boxed(finally_block, file, map, current_fn);
            }
        }
        Stmt::Export { item, .. } => walk_stmt(item, file, map, current_fn),
        _ => {}
    }
}

fn walk_boxed(stmt: &mut Box<Stmt>, file: &Path, map: &mut MarkMap, current_fn: Option<&str>) {
    walk_stmt(stmt, file, map, current_fn);
}

fn mark_stmt(id: usize, span: Span) -> Stmt {
    Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Identifier("__hudunit_mark".to_string(), span)),
        args: vec![Expr::Literal(
            Literal::String(id.to_string()),
            span,
        )],
        span,
    })
}

/// Per-file coverage result used by the reports.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileCoverage {
    pub path: String,
    pub executable_lines: Vec<usize>,
    pub covered_lines: Vec<usize>,
    #[serde(skip)]
    pub hit_counts: HashMap<usize, u64>,
}

impl FileCoverage {
    pub fn line_percent(&self) -> f64 {
        if self.executable_lines.is_empty() {
            return 100.0;
        }
        let covered = self
            .executable_lines
            .iter()
            .filter(|l| self.covered_lines.contains(l))
            .count();
        100.0 * covered as f64 / self.executable_lines.len() as f64
    }
}

/// Aggregate the marks into per-file line coverage. The test file's AST is
/// parsed from its own source (the prelude is a separate AST), so spans need
/// no offset adjustment. Each mark maps to its statement's START line only —
/// multi-line span ranges would over-report covered lines.
pub fn summarize(map: &MarkMap, marks: &Marks) -> Vec<FileCoverage> {
    let mut files: HashMap<PathBuf, (Vec<usize>, Vec<usize>, HashMap<usize, u64>)> =
        HashMap::new();
    for (id, info) in map.all() {
        let entry = files
            .entry(info.file.clone())
            .or_insert_with(|| (Vec::new(), Vec::new(), HashMap::new()));
        entry.0.push(info.start_line);
        let hits = marks.get(*id);
        if hits > 0 {
            entry.1.push(info.start_line);
            *entry.2.entry(info.start_line).or_insert(0) += hits;
        }
    }
    let mut out: Vec<FileCoverage> = files
        .into_iter()
        .map(|(path, (mut executable, mut covered, hit_counts))| {
            executable.sort_unstable();
            executable.dedup();
            covered.sort_unstable();
            covered.dedup();
            FileCoverage {
                path: path.display().to_string(),
                executable_lines: executable,
                covered_lines: covered,
                hit_counts,
            }
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// Function-level coverage summary: (function name, file, covered?).
/// A function counts as covered when any mark inside it (including its
/// entry mark) has been hit.
pub fn function_coverage(map: &MarkMap, marks: &Marks) -> Vec<(String, String, bool)> {
    let mut grouped: HashMap<(String, String), bool> = HashMap::new();
    for info in map.all().values() {
        let Some(name) = &info.function else { continue };
        let key = (name.clone(), info.file.display().to_string());
        let covered =
            grouped.get(&key).copied().unwrap_or(false) || marks.get(info.id) > 0;
        grouped.insert(key, covered);
    }
    let mut out: Vec<(String, String, bool)> = grouped
        .into_iter()
        .map(|((name, file), covered)| (name, file, covered))
        .collect();
    out.sort();
    out
}

/// Lexically collapse `.`/`..` segments so report paths read
/// `src/math.hud` instead of `tests/math/../../src/math.hud`.
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Module resolver that (a) resolves import paths relative to the importing
/// TEST file (the VM's default resolution is cwd-based, which breaks
/// `../../src/foo.hud`-style imports) and (b) instruments imported module
/// function bodies for coverage, formatting the AST back to source.
///
/// Instrumentation results are cached per canonical path so a module
/// imported by several test files keeps ONE stable set of mark ids. Any
/// failure (unparseable output, formatter trouble) falls back to the
/// untouched source — coverage must never break imports.
pub struct CoverageResolver {
    enabled: bool,
    mark_map: Arc<Mutex<MarkMap>>,
    cache: Arc<Mutex<HashMap<PathBuf, String>>>,
    base_dir: PathBuf,
}

impl CoverageResolver {
    pub fn new(
        enabled: bool,
        mark_map: Arc<Mutex<MarkMap>>,
        cache: Arc<Mutex<HashMap<PathBuf, String>>>,
        base_dir: PathBuf,
    ) -> Self {
        Self {
            enabled,
            mark_map,
            cache,
            base_dir,
        }
    }

    fn candidate(&self, path: &str) -> Option<PathBuf> {
        let given = Path::new(path);
        if given.is_file() {
            return Some(given.to_path_buf());
        }
        let joined = normalize_path(&self.base_dir.join(given));
        if joined.is_file() {
            return Some(joined);
        }
        // Extension-flexible lookup: `from "math"` may mean math.hud.
        for ext in ["hhs", "hud", "hudhud"] {
            let with_ext = normalize_path(&joined.with_extension(ext));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }
        None
    }

    fn instrumented_source(&self, file: &Path, source: &str) -> Option<String> {
        // Cache hit → ids were already allocated for this module.
        if let Some(hit) = self.cache.lock().unwrap().get(file) {
            return Some(hit.clone());
        }
        let mut ast = hudhudscript_parser::parse(source).ok()?;
        {
            let mut map = self.mark_map.lock().unwrap();
            instrument_module(&mut ast, file, &mut map);
        }
        let formatted = hudhudscript_formatter::Formatter::new().format_program(&ast);
        // The formatted source must round-trip through the parser — the VM's
        // module loader will parse it; a parse failure here means fallback.
        if hudhudscript_parser::parse(&formatted).is_err() {
            return None;
        }
        self.cache
            .lock()
            .unwrap()
            .insert(file.to_path_buf(), formatted.clone());
        Some(formatted)
    }
}

impl ModuleResolver for CoverageResolver {
    fn resolve(&self, path: &str, _from: Option<&str>) -> Result<ModuleContent, Error> {
        let file = self.candidate(path).ok_or_else(|| {
            hudhudscript_errors::runtime_error(format!("module '{}' not found", path))
        })?;
        let source = std::fs::read_to_string(&file).map_err(|e| {
            hudhudscript_errors::runtime_error(format!(
                "cannot read module '{}': {}",
                path, e
            ))
        })?;
        Ok(ModuleContent::Source(
            if self.enabled {
                self.instrumented_source(&file, &source).unwrap_or(source)
            } else {
                source
            },
        ))
    }

    fn exists(&self, path: &str) -> bool {
        self.candidate(path).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_seen_parses_ids() {
        let marks = Marks::new();
        marks.absorb_seen("3,7,12,");
        assert_eq!(marks.get(3), 1);
        assert_eq!(marks.get(7), 1);
        assert_eq!(marks.get(12), 1);
        assert_eq!(marks.get(99), 0);
    }

    #[test]
    fn absorb_seen_tolerates_garbage() {
        let marks = Marks::new();
        marks.absorb_seen("x,,3,y");
        assert_eq!(marks.get(3), 1);
    }

    #[test]
    fn instrument_marks_each_statement() {
        let mut ast = hudhudscript_parser::parse(
            "fn helper(x) {\n    return x + 1;\n}\nlet a = 1;\n",
        )
        .unwrap();
        let mut map = MarkMap::new();
        instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
        // helper body (entry + return) + top-level let = at least 3 marks
        assert!(map.all().len() >= 3);
        // Every mark call references the prelude helper
        let rendered = format!("{:?}", ast);
        assert!(rendered.contains("__hudunit_mark"));
    }

    #[test]
    fn prelude_helpers_not_instrumented() {
        let mut ast = hudhudscript_parser::parse(
            "fn __hudunit_mark(id) {\n    return 1;\n}\n",
        )
        .unwrap();
        let mut map = MarkMap::new();
        instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
        assert!(map.all().is_empty(), "prelude helpers must be skipped");
    }

    #[test]
    fn module_instrumentation_skips_top_level() {
        let mut ast = hudhudscript_parser::parse(
            "let a = 1;\nfn helper(x) {\n    return x + 1;\n}\n",
        )
        .unwrap();
        let mut map = MarkMap::new();
        instrument_module(&mut ast, Path::new("/t/src/mod.hud"), &mut map);
        // Only the function body (entry + return) got marks; `let a` did not.
        let top_level_marked = map
            .all()
            .values()
            .any(|info| info.start_line == 1);
        assert!(!top_level_marked, "module top level must stay unmarked");
        assert!(map.all().len() >= 2);
    }

    #[test]
    fn module_instrumentation_round_trips_through_formatter() {
        let source = "fn helper(x) {\n    if (x > 0) {\n        return x;\n    }\n    return -x;\n}\n";
        let mut ast = hudhudscript_parser::parse(source).unwrap();
        let mut map = MarkMap::new();
        instrument_module(&mut ast, Path::new("/t/src/mod.hud"), &mut map);
        let formatted = hudhudscript_formatter::Formatter::new().format_program(&ast);
        let reparsed = hudhudscript_parser::parse(&formatted)
            .expect("instrumented module must re-parse for the VM loader");
        assert!(!reparsed.is_empty());
    }

    #[test]
    fn summarize_reports_lines() {
        let mut ast =
            hudhudscript_parser::parse("fn helper() {\n    return 1;\n}\n").unwrap();
        let mut map = MarkMap::new();
        instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
        let marks = Marks::new();
        for id in map.all().keys() {
            marks.hit(*id);
        }
        let files = summarize(&map, &marks);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "/t/test_a.hud");
        assert!(files[0].line_percent() > 99.0);
    }
}
