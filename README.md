# HudHudScript

> Multilingual, register-based scripting language for AI orchestration, agent workflows, governance models, automation, and embeddable runtime use.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-{$VERSION}-blue.svg)](Cargo.toml)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

**Current version:** `{$VERSION}`

HudHudScript is a Rust-powered scripting language built around a register-based bytecode VM, native multilingual syntax, and first-class constructs for AI providers, agents, SOP (Subject-Oriented Programming), councils, swarms, and governance rules.

The goal is simple: let people model automation, AI systems, and authority flows in the language they think in, while still running on one consistent compiler, bytecode format, and VM.

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

| Area | Description |
|---|---|
| Native multilingual syntax | Write scripts with localized keywords and built-ins, including Turkish, Arabic, Japanese, English, Kurdish, Persian, and more. |
| Register-based VM | Bytecode is executed by a Rust VM designed around registers, direct instruction dispatch, compact values, and hot-path specialization. |
| AI-first language features | `provider`, `agent`, `swarm`, and pipeline-oriented patterns are part of the language model, not only external libraries. |
| Governance primitives | `role`, `council`, `community`, `constitution`, and `law` express authority, policy, membership, and voting directly. |
| SOP support | Subject-Oriented Programming concepts like `role`, `subject`, `effect`, and `view` are first-class modeling tools. |
| Runtime ecosystem | Compiler, VM, formatter, LSP, debugger, package/runtime crates, Python bindings, and plugin-facing standard library crates live in one workspace. |
| UI & Web Integration | Built-in Terminal UI (TUI) capabilities and comprehensive web/HTTP server operations for interactive and networked applications. |

HudHudScript is an AI-native, multilingual programming language that natively supports OOP, SOP (Subject-Oriented Programming), and Loop Engineering. With built-in constructs like MCP tooling, swarms, and councils, it serves as a powerful orchestration layer for AI products, local automation, agent systems, simulations, governance experiments, and embeddable domain runtimes.

By introducing AI-native paradigms, HudHudScript offers developers a unique and modern programming experience compared to traditional scripting languages. It provides native ways to model intelligent logic, orchestrate autonomous units, and govern software behaviors. For rigorous performance evaluations of this architecture, our comprehensive benchmark suites are publicly available at [HudHudMind/hudhudscript-benchmarks](https://github.com/HudHudMind/hudhudscript-benchmarks).

---

## Quick Example

```hudhud
// English
print("Hello, World!");

// Turkish
yazdır("Merhaba, Dünya!");

// Arabic
اطبع("مرحبا، عالم!");
```

All localized forms compile to the same kind of bytecode and run on the same VM.

```bash
hudhud run hello.hud
```

### Variables and Logic

```hudhud
let x = 10;
let y = 20;
let sum = x + y;
let ready = sum > 25;

if (ready) {
    print("Ready: " + sum);
} else {
    print("Not ready");
}
```

The same basic control flow can be written with native keywords:

| Language | let | if | else | print |
|---|---|---|---|---|
| Turkish | `değişken` | `eğer` | `değilse` | `yazdır` |
| Arabic | `متغير` | `إذا` | `وإلا` | `اطبع` |
| Russian | `пусть` | `если` | `иначе` | `print` |
| Chinese | `变量` | `如果` | `否则` | `print` |
| Japanese | `変数` | `もし` | `それでも` | `print` |

### AI Agent Pipeline

```hudhud
provider DeepSeek {
    type: "deepseek"
    model: "deepseek-chat"
    api_key: env("DEEPSEEK_API_KEY")
}

agent Translator {
    role: "translator"
    intent: "Translate text between languages"
    provider: DeepSeek
}

print("Agent Translator role:")
print(Translator["role"])
```

### SOP and Governance

```hudhud
role Mayor {
    can propose,
    can veto
}

role Treasurer {
    can budget,
    can audit
}

subject CityMayor has Mayor {
    state term: 1,
    state proposals: 0
}

subject CityTreasurer has Treasurer {
    state budget: 1000000,
    state spent: 0
}

council CityCouncil {
    quorum: 2
    members: ["CityMayor", "CityTreasurer"]
    rules: ["majority-vote"]
}
```

More examples are available in [`samples/`](samples/).

### Quick Samples

| Sample | Description |
|---|---|
| [`agent_pipeline.hud`](samples/agent_pipeline.hud) | Multi-agent pipeline |
| [`agent_provider.hud`](samples/agent_provider.hud) | AI provider declaration |
| [`city_council.hud`](samples/city_council.hud) | City council governance example |
| [`conditionals.hud`](samples/conditionals.hud) | If/else examples |
| [`exception_test.hud`](samples/exception_test.hud) | Exception flow sample |
| [`fibonacci.hud`](samples/fibonacci.hud) | Recursive computation |
| [`functions.hud`](samples/functions.hud) | Function declarations and calls |
| [`governance_council.hud`](samples/governance_council.hud) | Council role modeling |
| [`hello.hud`](samples/hello.hud) | Hello world |
| [`hello_world.hud`](samples/hello_world.hud) | Alternate hello world |
| [`loop_engineer.hud`](samples/loop_engineer.hud) | Loop engineering dry-run sample |
| [`loops.hud`](samples/loops.hud) | Loops and control flow |
| [`multilingual_print.hud`](samples/multilingual_print.hud) | Multilingual output |
| [`oop_class.hud`](samples/oop_class.hud) | Classes and methods |
| [`security_council.hud`](samples/security_council.hud) | Security council governance example |
| [`sop_arena.hud`](samples/sop_arena.hud) | SOP arena example |
| [`sop_ecommerce.hud`](samples/sop_ecommerce.hud) | SOP ecommerce model |
| [`sop_fleet.hud`](samples/sop_fleet.hud) | SOP fleet management |
| [`sop_inventory.hud`](samples/sop_inventory.hud) | SOP inventory model |
| [`sop_npc_rpg.hud`](samples/sop_npc_rpg.hud) | SOP game-style example |
| [`sop_subject.hud`](samples/sop_subject.hud) | Subject-Oriented Programming |
| [`swarm_council.hud`](samples/swarm_council.hud) | Swarm council coordination |
| [`tui_demo.hud`](samples/tui_demo.hud) | Terminal UI demo |

---

## Installation

### Build from Source

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
hudhud run script.hud        # Compile and execute
hudhud check script.hud      # Parse/check without running
hudhud compile script.hud    # Emit bytecode
hudhud fmt script.hud        # Format source code
hudhud repl                  # Interactive REPL
hudhud lsp                   # Language server
hudhud --version
hudhud --help
```

**File extensions:** `.hud`, `.hudhud`, and `.hhs` are supported.

---

## Language Features

### Core Syntax

- `let`, `const`, `var` declarations
- Arithmetic, comparison, boolean, and bitwise operators
- `if`, `else`, `while`, `for`, `loop`, `break`, `continue`
- Functions `fn`, closures, arrow functions
- Arrays, objects, destructuring, strings, numbers, booleans, null
- `try`, `catch`, `finally`
- Classes, inheritance, methods, `super`
- Async functions, await-style flows, promises
- Modules, imports, exports, package/runtime crates

### AI Agent Constructs

- `provider` AI backends: OpenAI, DeepSeek, Anthropic, Ollama, and custom providers
- `agent` role/intent driven AI units
- `swarm` coordinated multi-agent execution
- `council` voting governance workflows
- Tooling crates for MCP, resources, validation, sandboxing, RAG, package operations, and native extensions

### Governance and SOP

- `role` capability bundles
- `subject` entities with roles and state
- `effect` and `view` behavior composition
- `council`, `community`, `constitution`, `law` policy governance modeling
- Relation and rule systems for trust, influence, and authority graphs

### UI & Web Integration

- Built-in Terminal UI (TUI) components for interactive CLI applications
- Built-in web server capabilities
- HTTP client and server operations
- HTML templating and dynamic view evaluation

### Runtime Tooling

- Register-based bytecode VM
- Compact `Value16` runtime representation
- Compiler, parser, lexer, formatter, LSP, debugger, test crate, and CLI
- Python bindings through `hudhudscript-python`
- Benchmark regression tooling for VM/compiler performance work

---

## Project Structure

```text
hudhudscript/
├── Cargo.toml                 # Workspace root
├── crates/                    # Compiler, VM, CLI, tooling, stdlib crates
│   ├── hudhudscript-cli/      # `hudhud`, `hudc`, `hudi`, `hudp`
│   ├── hudhudscript-parser/
│   ├── hudhudscript-compiler/
│   ├── hudhudscript-bytecode/
│   ├── hudhudscript-vm/
│   ├── hudhudscript-lsp/
│   └── hudhud-*              # Standard library / extension crates
├── samples/                   # Compact demo programs
└── hudhudscript-tests/       # Integration test workspace copied for public release
```

---

## Building from Source

### Prerequisites

- Rust and Cargo
- Git
- Optional: Python 3.8+ for Python bindings
- Optional: Node.js for editor/website tooling

### Build

```bash
cargo build --workspace
cargo build --release -p hudhudscript-cli
```

### Test

```bash
cargo test --workspace
```

### Benchmark

```bash
cargo bench
./bench_release.sh
```

---

## Documentation

Documentation lives in [`docs/`](docs/) and examples live in [`samples/`](samples/). The source tree also contains specialized crates for parser, compiler, bytecode, VM, localization, package management, governance, sandboxing, validation, and AI/runtime integrations.

---

## Contributing

Issues, tests, documentation fixes, and benchmark reports are welcome. For performance work, include:

- exact commit and version
- benchmark source used
- release/debug mode
- median and p90 repeated runs
- bytecode opcode diff when applicable

---

## License

HudHudScript is licensed under the [MIT License](LICENSE).
