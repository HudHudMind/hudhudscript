# HudHudScript

> **Multi-lingual, register-based scripting language for AI orchestration, agent systems, and governance modeling.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)](Cargo.toml)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

HudHudScript is a programming language designed to be written in **your own language** — Turkish, Arabic, Japanese, English, Kurdish, Persian, and 18 more. Every keyword, every built-in function, every error message is localized natively. It runs on a **fully register-based VM** written in Rust, with first-class support for AI providers, agent pipelines, council/governance models, and SOPs (Standard Operating Procedures).

---

## Table of Contents

- [Why HudHudScript](#why-hudhudscript)
- [Quick Example](#quick-example)
- [Installation](#installation)
- [CLI Usage](#cli-usage)
- [Language Features](#language-features)
- [Project Structure](#project-structure)
- [Building from Source](#building-from-source)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

## Why HudHudScript

| | Description |
|---|---|
| 🌍 **24 Native Languages** | Write `yazdır("Merhaba")` in Turkish, `اطبع("مرحبا")` in Arabic, `表示("こんにちは")` in Japanese. Keywords and built-ins use the actual word in your language. |
| 🚀 **Register-Based VM** | Inspired by Lua 5.4. No stack/register hybrid — every local is a register. Fast, predictable execution. |
| 🤖 **AI-First Built-ins** | `provider`, `agent` are first-class constructs. Native integration with DeepSeek, OpenAI, Anthropic, Ollama, and local models. |
| 🏛️ **Governance Primitives** | `role`, `council`, `swarm`, `community`, `constitution`, `law` — model real-world authority systems directly. |
| 📜 **SOP Workflows** | Standard Operating Procedures as code — sequential, parallel, conditional flows with audit trails. |
| 🔐 **Sandbox & Validation** | Built-in capability-based sandbox, schema validation, MCP protocol bridge. |
| ⚡ **Multi-Frontend** | CLI (`hudhud`), Python module (`hudhudscript`), LSP server, REPL, web playground. |

---

## Quick Example

### Hello World — Three Ways

```hudhud
// English
print("Hello, World!")

// Türkçe
yazdır("Merhaba, Dünya!")

// العربية
اطبع("مرحبا، عالم!")
```

All three compile to the same bytecode. Run any of them:

```bash
hudhud run hello.hud
```

### Variables & Logic

```hudhud
// English
let x = 10
let y = 20
let sum = x + y
let is_ready = true
let is_valid = sum > 25 && is_ready
if (is_valid) { print("Ready: " + sum) } else { print("Not ready") }
```

Same logic in 5 languages (every keyword in its native form):

| Dil | let | if | else | print |
|---|---|---|---|---|
| Türkçe | `değişken` | `eğer` | `değilse` | `yazdır` |
| العربية | `متغير` | `إذا` | `وإلا` | `اطبع` |
| Русский | `пусть` | `если` | `иначе` | `print` |
| 中文 | `变量` | `如果` | `否则` | `print` |
| 日本語 | `変数` | `もし` | `それでも` | `print` |

```hudhud
// Türkçe
değişken x = 10; değişken y = 20; değişken t = x + y
eğer (t > 25) { yazdır("Evet") } değilse { yazdır("Hayır") }

// العربية
متغير x = 10; متغير y = 20; متغير t = x + y
إذا (t > 25) { اطبع("نعم") } وإلا { اطبع("لا") }

// Русский
пусть x = 10; пусть y = 20; пусть t = x + y
если (t > 25) { print("Да") } иначе { print("Нет") }

// 中文
变量 x = 10; 变量 y = 20; 变量 t = x + y
如果 (t > 25) { print("是") } 否则 { print("否") }

// 日本語
変数 x = 10; 変数 y = 20; 変数 t = x + y
もし (t > 25) { print("はい") } それでも { print("いいえ") }
```

### AI Agent Pipeline

```hudhud
provider DeepSeek {
    type: "deepseek",
    model: "deepseek-chat",
    api_key: env("DEEPSEEK_API_KEY"),
    temperature: 0.7
}

agent Translator {
    role: "translator",
    intent: "Translate text between languages"
}

fn translate(prov, text, target_lang) {
    let prompt = "Translate to " + target_lang + ": " + text
    return prov.call({ prompt: prompt })
}

let result = translate(DeepSeek, "Hello world", "Turkish")
print(result)
```

### SOP — Subject-Oriented Programming

```hudhud
role Fighter { can attack, can defend }
subject Knight has Fighter {
    state health: 100, state power: 30
}
effect on Damage(target, amount) { target.health = target.health - amount }
on attack(self, target) { Damage(target, self.power) }
let hero = spawn Knight; let boss = spawn Knight
hero.attack(boss)
print("Boss HP: "); print(boss.health)
```

### Council & Governance

```hudhud
role Mayor      { can propose, can veto }
role Treasurer  { can budget, can audit }
role Sheriff    { can enforce, can detain }

subject CityMayor      has Mayor      { state term: 1 }
subject CityTreasurer  has Treasurer  { state budget: 1_000_000 }
subject CitySheriff    has Sheriff    { state officers: 50 }

council CityCouncil {
    quorum: 2,
    members: ["CityMayor", "CityTreasurer", "CitySheriff"],
    rules: ["majority-vote", "public-record"]
}

spawn CityMayor; spawn CityTreasurer; spawn CitySheriff;
```

More samples in [`samples/`](samples/).

### Quick Samples

All samples in [`samples/`](samples/):

| Sample | Description |
|---|---|
| **SOP** ||
| [`sop_subject.hud`](samples/sop_subject.hud) | Subject-Oriented Programming — roles, effects, views |
| [`sop_npc_rpg.hud`](samples/sop_npc_rpg.hud) | NPC RPG — full game loop |
| [`sop_arena.hud`](samples/sop_arena.hud) | Arena Combat — of, compose, despawn |
| [`sop_inventory.hud`](samples/sop_inventory.hud) | SOP + OOP hybrid — stock management |
| [`sop_ecommerce.hud`](samples/sop_ecommerce.hud) | SOP E-Commerce |
| [`sop_fleet.hud`](samples/sop_fleet.hud) | SOP Fleet management |
| **OOP** ||
| [`oop_class.hud`](samples/oop_class.hud) | Classes, abstract, inheritance, methods |
| **Agent + Provider** ||
| [`agent_provider.hud`](samples/agent_provider.hud) | Agent with DeepSeek provider |
| [`agent_pipeline.hud`](samples/agent_pipeline.hud) | Multi-agent pipeline |
| **Governance** ||
| [`governance_council.hud`](samples/governance_council.hud) | Council, Swarm, Roles, Subjects |
| [`city_council.hud`](samples/city_council.hud) | City council meeting |
| [`swarm_council.hud`](samples/swarm_council.hud) | Swarm + Council + Community |
| [`security_council.hud`](samples/security_council.hud) | Security council |
| **Multilingual** ||
| [`multilingual_print.hud`](samples/multilingual_print.hud) | 26 dilde print() |
| **TUI** ||
| [`tui_demo.hud`](samples/tui_demo.hud) | Terminal UI — counter demo |
| **Basics** ||
| [`hello.hud`](samples/hello.hud) | Hello World |
| [`functions.hud`](samples/functions.hud) | Functions |
| [`loops.hud`](samples/loops.hud) | Loops & conditionals |
| [`conditionals.hud`](samples/conditionals.hud) | If/else, ternary |
| [`fibonacci.hud`](samples/fibonacci.hud) | Fibonacci |

---

## Installation

### From Source

```bash
git clone https://github.com/HudHudMind/hudhudscript.git
cd hudhudscript
cargo build --release -p hudhudscript-cli
./target/release/hudhud --version
```

---

## CLI Usage

The main binary is `hudhud`.

```bash
hudhud run    script.hud      # Compile + execute
hudhud check  script.hud      # Type-check + lint, no execution
hudhud compile script.hud     # Emit bytecode
hudhud fmt    script.hud      # Format source code
hudhud repl                   # Interactive REPL
hudhud lsp                    # Start LSP server (for editor integration)
hudhud --version
hudhud --help
```

**File extensions:** `.hud`, `.hudhud`, `.hhs` — all valid.

---

## Language Features

### Core Syntax

- `let`, `const`, `var` declarations
- Arithmetic, boolean, comparison, bitwise operators
- `if` / `else` / `while` / `for` / `loop` / `break` / `continue`
- Functions (`fn`), closures, arrow functions
- Arrays, objects (maps), destructuring
- `try` / `catch` / `finally`
- Classes & OOP (`class`, `extends`, `super`)
- Async / await (`async fn`, `.then()`, `.await`)
- Modules (`use`, `import`, `export`)

All keywords have native equivalents in 24 languages — see the Variables & Logic example above.

### AI Orchestration Constructs

- `provider` — declare an AI backend (OpenAI, DeepSeek, Anthropic, Ollama, custom)
- `agent` — AI agent with role, intent, and provider binding
- `swarm` — parallel agent execution group
- `council` — voting body with governance rules

### Governance & SOP

- `role` — capability bundle
- `subject` — entity that holds roles
- `council` — voting body
- `swarm` — leaderless collective
- `community` — membership graph
- `constitution`, `law` — rule definitions
- `relation` — trust/influence edges
- `sop` — Subject-Oriented Programming (Harrison & Ossher)

### Built-in Crates

Over 60 utility crates ship in [`crates/`](crates/) prefixed `hudhud-*`:

```
hudhud-http     hudhud-fs        hudhud-net       hudhud-crypto
hudhud-pdf      hudhud-ocr       hudhud-email     hudhud-regex
hudhud-math     hudhud-linalg    hudhud-datetime  hudhud-archive
hudhud-docker   hudhud-firewall  hudhud-gpu       hudhud-hardware
hudhud-media    hudhud-print     hudhud-translate hudhud-workflow
... and more
```

---

## Project Structure

```
hudhudscript/
├── Cargo.toml              # Workspace root
├── crates/                 # 90+ Rust crates (compiler, VM, stdlib, tools)
│   ├── hudhudscript-cli/   # `hudhud` binary
│   ├── hudhudscript-lexer/
│   ├── hudhudscript-parser/
│   ├── hudhudscript-compiler/
│   ├── hudhudscript-vm/    # Register-based VM
│   ├── hudhudscript-runtime/
│   ├── hudhudscript-lsp/   # Editor language server
│   ├── hudhudscript-python/ # PyO3 bindings
│   └── hudhud-*            # Standard library crates
├── examples/               # Curated example programs
│   ├── 01-basics/
│   ├── 02-multilingual/
│   ├── 03-council/
│   ├── 04-agents/
│   └── 05-advanced/
├── samples/                # Compact one-file demos
├── docs/                   # Language reference, book, paper
├── editors/                # VS Code, Vim, Emacs plugins
├── installer/              # Platform-specific installers
├── benches/                # Performance benchmarks
└── hudhudscript-tests/    # Cross-crate integration tests
```

---

## Building from Source

### Prerequisites

- **Rust 1.93+** ([install](https://rustup.rs/))
- **Cargo** (ships with Rust)
- **Git**
- *(Optional)* **Python 3.8+** + **maturin** for the Python module
- *(Optional)* **Node.js 20+** for editor plugins

### Clone & Build

```bash
git clone https://github.com/HudHudMind/hudhudscript.git
cd hudhudscript

# Debug build (fast compile, slow runtime)
cargo build -p hudhudscript-cli

# Release build (slow compile, fast runtime)
cargo build --release -p hudhudscript-cli

# The binary lands at target/release/hudhud
./target/release/hudhud --version
```

### Add to PATH

```bash
sudo cp target/release/hudhud /usr/local/bin/
hudhud run examples/01-basics/hello_world.hud
```

### Build the Python Module

```bash
cd crates/hudhudscript-python
maturin develop --release
python3 -c "import hudhudscript; print(hudhudscript.version())"
```

### Run Tests

```bash
# All tests
cargo test --workspace

# VM parity tests
cargo test --test vm_parity_tests

# Single crate
cargo test -p hudhudscript-vm
```

### Run Benchmarks

```bash
cargo bench
# Or:
./bench_release.sh
```

---

## Documentation

Full documentation, language reference, and guides are available on our website:

| Resource | Link |
|---|---|
| **Official Website** | [hudhudscript.com](https://hudhudscript.com) |
| **Language Reference** | [hudhudscript.com/docs](https://hudhudscript.com/docs) |
| **API Reference** | [hudhudscript.com/api](https://hudhudscript.com/api) |
| **Examples & Tutorials** | [hudhudscript.com/examples](https://hudhudscript.com/examples) |

For implementation details, see the source code and comments in [`crates/`](crates/).

---

## Contributing

HudHudScript follows a strict project constitution (see [`CLAUDE.md`](CLAUDE.md) and [`AGENTS.md`](AGENTS.md)):

1. **Test sanctity** — approved unit tests are immutable; only new tests can be added.
2. **VM is the single runtime** — no interpreter fallback, no hybrid stack/register designs.
3. **Single Source of Truth** — no parallel implementations, no duplicated code, no "alternative" systems.
4. **Implementation = Integration** — every new struct/trait must be wired into production runtime in the same commit.
5. **400-line file limit** — files larger than 400 lines must be refactored, no exceptions.

### Reporting Issues

- Open an issue on [GitHub](https://github.com/HudHudMind/hudhudscript/issues)
- Or report via the website's error reporter
- Include: HudHudScript version (`hudhud --version`), OS, minimal reproducing script

### Pull Requests

- Run `cargo test --workspace` before submitting
- Run `cargo clippy --workspace --all-targets -- -D warnings`
- Run `cargo fmt --all`
- Add tests for new functionality
- Update relevant documentation

---

## License

MIT License © 2024 Onur GÜZEL — see [`LICENSE`](LICENSE) for details.

---

## Acknowledgments

HudHudScript is named after the **Hudhud** (hoopoe) — the bird mentioned in classical literature as a messenger and bridge between worlds. The language aims to be exactly that: a bridge between human languages, between human intent and machine execution, between AI agents and the systems they serve.
