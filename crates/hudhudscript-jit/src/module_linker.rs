//! JIT modül birleştirici (M4, v0.9.37) — AST-düzeyi import linkleme.
//!
//! ES-tarzı `import { a, b } from "./mod.hud"` deyimleri çözümlenir:
//! hedef dosya parse edilir, İSTENEN adlı fonksiyon tanımları ana moda
//! eklenir, import deyimi kaldırılır. Özyinelemelidir (iç içe importlar,
//! derinlik sınırı 16); döngüsel import küme kontrolüyle reddedilir.
//!
//! Dürüst-fallback sözleşmesi: çözümlenemeyen her durum (ağ şemalı yol,
//! default/wildcard import, ad çakışması, dosya yok) Err döner →
//! precheck/lowering hatası → bilinen VM-fallback yolu. Asla yanlış kod.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use hudhudscript_ast::ImportKind;
use hudhudscript_ast::Stmt;
use hudhudscript_modules::ModuleResolver;

const MAX_DEPTH: usize = 16;

/// Import deyimlerini çözüp tek stmt listesine birleştirir.
/// `base_dir`: göreli importların çözümleneceği dizin (None → import yok sayılır hata olarak kalır).
pub fn link_imports(
    stmts: &[Stmt],
    base_dir: Option<&Path>,
) -> Result<Vec<Stmt>, String> {
    let mut out = Vec::with_capacity(stmts.len());
    let mut visited: HashSet<PathBuf> = HashSet::new();
    link_stmts(stmts, base_dir, &mut visited, 0, &mut out)?;
    Ok(out)
}

fn link_stmts(
    stmts: &[Stmt],
    base_dir: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
    out: &mut Vec<Stmt>,
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("module import depth exceeds 16 (cycle?)".into());
    }
    for stmt in stmts {
        match stmt {
            Stmt::Import { path, imports, .. } => {
                let dir = base_dir.ok_or_else(|| {
                    format!("import '{path}' needs a base directory (JitRuntime::set_base_dir)")
                })?;
                let names = named_imports(imports)?;
                let resolved = resolve_local(dir, path)?;
                if !visited.insert(resolved.clone()) {
                    return Err(format!("cyclic module import: {path}"));
                }
                let src = std::fs::read_to_string(&resolved)
                    .map_err(|e| format!("import '{path}': {e}"))?;
                let ast = hudhudscript_parser::parse(&src)
                    .map_err(|e| format!("import '{path}': {e}"))?;
                // Modülün kendi importları da çözülür (aynı dizin bazlı)
                let mut inner: Vec<Stmt> = Vec::new();
                link_stmts(&ast, resolved.parent(), visited, depth + 1, &mut inner)?;
                for name in &names {
                    let found = inner.iter().any(|s| matches!(s, Stmt::Function { name: n, .. } if n == name));
                    if !found {
                        return Err(format!("import '{path}': '{name}' not exported"));
                    }
                }
                for s in inner {
                    if let Stmt::Function { name, .. } = &s {
                        if names.contains(name) {
                            out.push(s);
                        }
                    } else {
                        // Modül-düzeyi let/global: VM paritesinde paylaşılan
                        // durum — JIT şeridinde dışarı sızmaz, yalnız fn'ler alınır.
                    }
                }
            }
            other => out.push(other.clone()),
        }
    }
    Ok(())
}

fn named_imports(kind: &ImportKind) -> Result<Vec<String>, String> {
    match kind {
        ImportKind::Named(items) => Ok(items.clone()),
        ImportKind::Default(_) => Err("default imports arrive later (VM fallback)".into()),
        ImportKind::Wildcard(_) => Err("wildcard imports arrive later (VM fallback)".into()),
    }
}

fn resolve_local(dir: &Path, path: &str) -> Result<PathBuf, String> {
    if path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("plugin:")
        || path.starts_with("node:")
    {
        return Err(format!("non-local import '{path}' requires VM module resolver"));
    }
    let mut resolver = ModuleResolver::new(dir.to_path_buf());
    resolver.resolve(path).map_err(|e| format!("import '{path}': {e}"))
}
