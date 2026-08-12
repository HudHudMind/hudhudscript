//! Loop engineering compile-time symbol/validation (FAZ C)
//! Uses AST types only — no orchestration runtime model dependencies.
use hudhudscript_ast::*;
use std::collections::{HashMap, HashSet};

/// Compile-time symbol table for loop engineering declarations.
pub struct LoopSymbolTable {
    pub loops: HashMap<String, LoopSymbol>,
    pub chains: HashMap<String, ChainSymbol>,
    pub errors: Vec<String>,
}

pub struct LoopSymbol {
    pub name: String,
    pub steps: Vec<String>,
    pub gates: Vec<String>,
    pub targets: Vec<GateTargetAst>,
}

pub struct ChainSymbol {
    pub name: String,
    pub link_names: Vec<String>,
}

/// Collect symbols from parsed AST statements. Pure compile-time — no runtime deps.
pub fn collect_loop_symbols(stmts: &[Stmt]) -> LoopSymbolTable {
    let mut table = LoopSymbolTable {
        loops: HashMap::new(), chains: HashMap::new(), errors: Vec::new(),
    };

    for stmt in stmts {
        if let Stmt::Decl(decl) = stmt {
            match decl {
                Decl::Loop { name, items, .. } => {
                    if table.loops.contains_key(name.as_str()) {
                        table.errors.push(format!("duplicate loop: '{}'", name));
                        continue;
                    }
                    let mut steps = Vec::new();
                    let mut gates = Vec::new();
                    let mut targets = Vec::new();
                    for item in items {
                        match item {
                            LoopItemAst::InlineStep(s) => {
                                if let Decl::Step { name: sname, gate, .. } = s.as_ref() {
                                    steps.push(sname.clone());
                                    if let Some(g) = gate {
                                        gates.push(g.name.clone());
                                        for b in &g.branches { targets.push(b.target.clone()); }
                                        targets.push(g.else_target.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    table.loops.insert(name.clone(), LoopSymbol { name: name.clone(), steps, gates, targets });
                }
                Decl::Chain { name, links, .. } => {
                    if table.chains.contains_key(name.as_str()) {
                        table.errors.push(format!("duplicate chain: '{}'", name));
                        continue;
                    }
                    let link_names: Vec<String> = links.iter().map(|l| l.loop_name.clone()).collect();
                    table.chains.insert(name.clone(), ChainSymbol { name: name.clone(), link_names });
                    // Collect nested inline loop declarations inside chain links
                    for link in links {
                        if let Some(inline) = &link.inline_loop {
                            if let Decl::Loop { name: ln, items, .. } = inline.as_ref() {
                                if table.loops.contains_key(ln.as_str()) {
                                    table.errors.push(format!("duplicate loop: '{}'", ln));
                                    continue;
                                }
                                let mut steps = Vec::new();
                                let mut gates = Vec::new();
                                let mut targets = Vec::new();
                                for item in items {
                                    if let LoopItemAst::InlineStep(s) = item {
                                        if let Decl::Step { name: sname, gate, .. } = s.as_ref() {
                                            steps.push(sname.clone());
                                            if let Some(g) = gate {
                                                gates.push(g.name.clone());
                                                for b in &g.branches { targets.push(b.target.clone()); }
                                                targets.push(g.else_target.clone());
                                            }
                                        }
                                    }
                                }
                                table.loops.insert(ln.clone(), LoopSymbol { name: ln.clone(), steps, gates, targets });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    table
}

/// Validate cross-loop references, chain link existence, and duplicate names.
pub fn validate_loop_symbols(table: &LoopSymbolTable) -> Vec<String> {
    let mut errors = Vec::new();
    let loop_names: HashSet<&str> = table.loops.keys().map(|s| s.as_str()).collect();

    // Validate cross-loop targets
    for (name, sym) in &table.loops {
        for target in &sym.targets {
            match target {
                GateTargetAst::Loop(ln) => {
                    if !loop_names.contains(ln.as_str()) {
                        errors.push(format!("loop '{}': unknown target loop '{}'", name, ln));
                    }
                }
                GateTargetAst::LoopStep(ln, sn) => {
                    if !loop_names.contains(ln.as_str()) {
                        errors.push(format!("loop '{}': unknown target loop '{}'", name, ln));
                    } else if let Some(target_loop) = table.loops.get(ln) {
                        if !target_loop.steps.contains(sn) {
                            errors.push(format!("loop '{}': target loop '{}' has no step '{}'", name, ln, sn));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Validate chain links
    for (name, sym) in &table.chains {
        for link in &sym.link_names {
            if !loop_names.contains(link.as_str()) {
                errors.push(format!("chain '{}': unknown link loop '{}'", name, link));
            }
        }
    }

    errors
}
