//! Parser performance benchmarks (Issue #32)
//!
//! Measures Pest grammar parsing time for a representative set of
//! HudHudScript constructs: simple expressions, function declarations,
//! agent declarations, and governance blocks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hudhudscript_parser::parse;

// ---------------------------------------------------------------------------
// Benchmark inputs
// ---------------------------------------------------------------------------

const SIMPLE_EXPRESSION: &str = r#"let x = 1 + 2 * 3;"#;

const VARIABLE_ASSIGNMENT: &str = r#"
let a = 42;
let b = "hello";
let c = true;
let d = a + b;
"#;

const FUNCTION_SIMPLE: &str = r#"
fn add(a, b) {
    return a + b;
}
"#;

const FUNCTION_COMPLEX: &str = r#"
fn fibonacci(n) {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn factorial(n) {
    if n == 0 {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;

const AGENT_SIMPLE: &str = r#"
agent MyAgent {
    model: "gpt-4",
    task: "Summarize the given text"
}
"#;

const AGENT_WITH_TOOLS: &str = r#"
agent ResearchAgent {
    model: "gpt-4o",
    task: "Research the given topic thoroughly",
    max_iterations: 10
}
"#;

const PROVIDER_DECLARATION: &str = r#"
provider openai_gpt4 {
    type: "openai",
    model: "gpt-4",
    api_key: "sk-test-key",
    max_tokens: 4000
}
"#;

const GOVERNANCE_CONSTITUTION: &str = r#"
constitution DataGovernance {
    description: "Data handling rules",
    laws: [
        {
            name: "Privacy Law",
            description: "Protect user privacy",
            enforcement: mandatory,
            rules: ["rule.1", "rule.2"]
        }
    ]
}
"#;

const GOVERNANCE_LAW: &str = r#"
law PrivacyLaw {
    name: "Privacy Protection",
    description: "Ensure user data privacy",
    enforcement: mandatory,
    rules: ["no_pii_logging", "encrypt_at_rest", "access_audit"]
}
"#;

const IMPORT_STATEMENT: &str = r#"use postgres as db;"#;

const MULTILANG_TURKISH: &str = r#"
postgres sunucusunu db olarak kullan;
"#;

const MULTILANG_JAPANESE: &str = "postgres を db として 使う;\n";

/// A realistic mixed program combining multiple constructs.
const MIXED_PROGRAM: &str = r#"
use postgres as db;
use openai as ai;

provider openai_main {
    type: "openai",
    model: "gpt-4o",
    api_key: "sk-key",
    max_tokens: 8000
}

constitution AppPolicy {
    description: "Application governance",
    laws: [
        {
            name: "Data Safety",
            description: "Keep data safe",
            enforcement: mandatory,
            rules: ["rule.encrypt", "rule.audit"]
        }
    ]
}

fn process_query(query, context) {
    let result = db.query(query);
    if result {
        return result;
    }
    return null;
}

agent DataAgent {
    model: "gpt-4o",
    task: "Answer questions about database content"
}
"#;

/// Repeated simple expressions to stress-test throughput.
fn make_repeated_expressions(n: usize) -> String {
    (0..n)
        .map(|i| format!("let var_{i} = {i} + {i};"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

fn bench_simple_expressions(c: &mut Criterion) {
    c.bench_function("parse/simple_expression", |b| {
        b.iter(|| parse(black_box(SIMPLE_EXPRESSION)).unwrap())
    });

    c.bench_function("parse/variable_assignment", |b| {
        b.iter(|| parse(black_box(VARIABLE_ASSIGNMENT)).unwrap())
    });
}

fn bench_function_declarations(c: &mut Criterion) {
    c.bench_function("parse/function_simple", |b| {
        b.iter(|| parse(black_box(FUNCTION_SIMPLE)).unwrap())
    });

    c.bench_function("parse/function_complex", |b| {
        b.iter(|| parse(black_box(FUNCTION_COMPLEX)).unwrap())
    });
}

fn bench_agent_declarations(c: &mut Criterion) {
    c.bench_function("parse/agent_simple", |b| {
        b.iter(|| parse(black_box(AGENT_SIMPLE)).unwrap())
    });

    c.bench_function("parse/agent_with_tools", |b| {
        b.iter(|| parse(black_box(AGENT_WITH_TOOLS)).unwrap())
    });

    c.bench_function("parse/provider_declaration", |b| {
        b.iter(|| parse(black_box(PROVIDER_DECLARATION)).unwrap())
    });
}

fn bench_governance_blocks(c: &mut Criterion) {
    c.bench_function("parse/governance_constitution", |b| {
        b.iter(|| parse(black_box(GOVERNANCE_CONSTITUTION)).unwrap())
    });

    c.bench_function("parse/governance_law", |b| {
        b.iter(|| parse(black_box(GOVERNANCE_LAW)).unwrap())
    });
}

fn bench_multilang(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse/multilang");
    group.bench_function("english", |b| {
        b.iter(|| parse(black_box(IMPORT_STATEMENT)).unwrap())
    });
    group.bench_function("turkish", |b| {
        b.iter(|| parse(black_box(MULTILANG_TURKISH)).unwrap())
    });
    group.bench_function("japanese", |b| {
        b.iter(|| parse(black_box(MULTILANG_JAPANESE)).unwrap())
    });
    group.finish();
}

fn bench_mixed_program(c: &mut Criterion) {
    c.bench_function("parse/mixed_program", |b| {
        b.iter(|| parse(black_box(MIXED_PROGRAM)).unwrap())
    });
}

fn bench_throughput_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse/scaling");
    for &n in &[10usize, 50, 100, 200] {
        let source = make_repeated_expressions(n);
        group.bench_with_input(
            BenchmarkId::new("repeated_expressions", n),
            &source,
            |b, s| b.iter(|| parse(black_box(s.as_str())).unwrap()),
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry points
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_simple_expressions,
    bench_function_declarations,
    bench_agent_declarations,
    bench_governance_blocks,
    bench_multilang,
    bench_mixed_program,
    bench_throughput_scaling,
);
criterion_main!(benches);
