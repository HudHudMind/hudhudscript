pub use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::prepack::{prepack_instructions, PACK_SENTINEL};
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use crate::vm::registry::{BuiltinFn, ModuleMethodHandler, ModuleRegistry};
use crate::vm::VM;
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::packed_instruction;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{
    Bytecode, ClassData, FunctionChunk, FunctionData, GeneratorState16, InstanceData, Instruction,
    PromiseState16, Value16,
};
use hudhudscript_debug::Debugger;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::{Condition, Constitution};
use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
use hudhudscript_rag::{
    DistanceMetric, EmbeddingProvider, SimpleEmbedding, VectorStore, VectorStoreConfig,
};
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            last_return: Value16::null(),
            globals: rustc_hash::FxHashMap::with_capacity_and_hasher(100, Default::default()),
            shared_globals_vec: Vec::new(),
            scope_cells: Vec::new(),
            scope_cells_pool: Vec::new(),
            locale: OutputLocale::Default,
            try_frames: Vec::new(),
            finally_frames: Vec::new(),
            pending_flow: None,
            iterators: Vec::new(),
            iterator_generators: Vec::new(),
            loop_headers: Vec::new(),
            classes: rustc_hash::FxHashMap::with_capacity_and_hasher(16, Default::default()),
            declarations: rustc_hash::FxHashMap::with_capacity_and_hasher(8, Default::default()),
            effects: rustc_hash::FxHashMap::with_capacity_and_hasher(8, Default::default()),
            relations: rustc_hash::FxHashMap::with_capacity_and_hasher(8, Default::default()),
            subject_templates: rustc_hash::FxHashMap::with_capacity_and_hasher(
                4,
                Default::default(),
            ),
            event_schemas: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            subject_instances: rustc_hash::FxHashMap::with_capacity_and_hasher(
                4,
                Default::default(),
            ),
            composition_rules: rustc_hash::FxHashMap::with_capacity_and_hasher(
                4,
                Default::default(),
            ),
            field_correspondences: rustc_hash::FxHashMap::with_capacity_and_hasher(
                4,
                Default::default(),
            ),
            toml_providers: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            toml_config: Value16::null(),
            dispatch_provider_receiver: None,
            agent_names: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            swarm_names: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            council_names: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            community_names: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            ability_handlers: rustc_hash::FxHashMap::with_capacity_and_hasher(
                4,
                Default::default(),
            ),
            constitutions: rustc_hash::FxHashMap::with_capacity_and_hasher(4, Default::default()),
            active_constitution: None,
            provider: None,
            provider_registry: None,
            tool_registry: None,
            mcp_tool_definitions: Arc::new(Mutex::new(HashMap::new())),
            mcp_clients: Arc::new(Mutex::new(HashMap::new())),
            call_depth: 0,
            max_call_depth: hudhudscript_errors::constants::MAX_CALL_DEPTH,
            max_call_depth_hard_ceiling: 4000,
            // FIX-1: keep arena tiny at startup; it grows on demand.
            // A 256K-slot pre-grow caused a 4MB memset + page-fault tax on
            // every short script run, dwarfing actual VM execution time.
            registers: crate::vm::register_arena::RegisterArena::with_capacity(256),
            call_stack_local_syms: Vec::with_capacity(64),
            owned_local_sym_refs: Vec::with_capacity(16),
            pending_call: None,
            pending_super_call: false,
            frame_stack: Vec::with_capacity(64),
            stack_frame_base: 0,
            fuel_limit: None,
            register_arena_size: 64 * 1024,
            mailbox_capacity: hudhudscript_errors::constants::MAILBOX_CAPACITY,
            max_mcp_servers: 128,
            max_builtin_iter: 10_000,
            default_stack_bytes: 8 * 1024 * 1024,
            fuel_remaining: 0,
            sandbox: Some(Box::new(SandboxConfig {
                allowed_paths: vec![],
                allowed_hosts: vec![],
                allow_file_read: true,
                allow_file_write: false,
                allow_network: false,
                allow_process: false,
                allowed_commands: vec![],
                denied_commands: vec![
                    "rm".into(),
                    "dd".into(),
                    "mkfs".into(),
                    "shutdown".into(),
                    "reboot".into(),
                ],
            })),
            host_access_policy: crate::vm::host_access::HostAccessPolicy::permissive(),
            tvars: hudhudscript_stm::TVarRegistry::new(),
            class_context_stack: Vec::new(),
            last_instance_mutation: None,
            immutables: HashSet::new(),
            sym_cache: rustc_hash::FxHashMap::with_capacity_and_hasher(128, Default::default()),
            name_sym_cache: std::cell::RefCell::new(
                rustc_hash::FxHashMap::with_capacity_and_hasher(128, Default::default()),
            ),
            length_sym_id: hudhudscript_bytecode::interner::intern("length").0,
            push_sym_id: hudhudscript_bytecode::interner::intern("push").0,
            math_obj: None,
            json_obj: None,
            in_stm_context: false,
            current_tx: None,

            debugger: None,
            current_file: None,
            actors: Arc::new(hudhudscript_bytecode::actor_registry::SharedActorRegistry::new()),
            actor_mailboxes: HashMap::new(),
            promise_registry: hudhudscript_async::PromiseRegistry::new(),
            rag_embedder: SimpleEmbedding::new(128)
                .unwrap_or_else(|_| SimpleEmbedding::new(64).unwrap()),
            rag_stores: rustc_hash::FxHashMap::with_capacity_and_hasher(8, Default::default()),
            rag_text_to_ids: HashMap::new(),
            call_stack_names: Vec::with_capacity(64),
            module_registry: ModuleRegistry::new(),
            module_resolver: None,
            execution_deadline: None,
            cancellation_token: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            suspended_ip: None,
            tco_args: None,
            args_scratch: Vec::with_capacity(32),
            yield_sender: None,
            yield_receivers: rustc_hash::FxHashMap::default(),
            detached_promises: rustc_hash::FxHashMap::default(),
            next_yield_id: 0,
            chunk_cache: rustc_hash::FxHashMap::with_capacity_and_hasher(64, Default::default()),
            chunk_cache_last_key: 0,
            chunk_cache_last_val: None,
            current_chunk_ptr: std::ptr::null(),
            str_chars_cache: rustc_hash::FxHashMap::with_capacity_and_hasher(8, Default::default()),
            main_local_slots: Vec::new(),
            gc_constant_roots: Vec::new(),
            constant_root_chunks: rustc_hash::FxHashSet::default(),
            call_cache: Vec::with_capacity(64),
            ability_cache: [(None, 0), (None, 0)],
        };
        vm.register_globals();
        vm
    }

    /// Register an external module handler (#928).
    /// The handler will be called for any method invocation on objects whose
    /// `__module` field matches `name`, taking priority over hardcoded dispatch.
    /// Register a single builtin method under `module::method`.
    pub fn register_method(&mut self, module: &str, method: &str, handler: BuiltinFn) {
        self.module_registry
            .register_method(module, method, handler);
    }

    /// Register a legacy module-wide handler (transitioning to per-method fns).
    pub fn register_module(&mut self, name: &str, handler: ModuleMethodHandler) {
        self.module_registry.register_module(name, handler);
    }

    /// Create VM with specific locale
    pub fn with_locale(locale: OutputLocale) -> Self {
        let mut vm = Self::new();
        vm.locale = locale;
        vm
    }

    /// Set an external module resolver for LoadModule instruction (#921).
    pub fn set_module_resolver(&mut self, resolver: Box<dyn hudhudscript_errors::ModuleResolver>) {
        self.module_resolver = Some(resolver);
    }

    // ── Fuel/Gas API ─────────────────────────────────────────────────

    /// Set execution fuel limit (in-place `&mut self` setter).  Kept as
    /// `with_fuel` — the original method name preserved coverage-tests
    /// expect (`vm.with_fuel(100)`).  The interpreter-parity builder

    /// If a try-frame is active, restore VM state to the TryBegin snapshot,
    /// push the error as a catchable value on the stack (for the catch
    /// param binding), and return the catch IP.
    ///
    /// Returns `None` if no try-frame is active — caller should propagate
    /// the original error up.
    ///
    /// This mirrors the interpreter's `execute_try` behaviour
    /// (`crates/hudhudscript-interpreter/src/execution/control_flow.rs`
    /// lines 316-374) — every runtime error that is NOT a control-flow
    /// signal is convertible to a catchable value.

    // ── Sandbox API ─────────────────────────────────────────────────

    /// Set sandbox configuration on an existing VM.  The VM's local
    /// `SandboxConfig` is a flattened subset of the full
    /// `hudhudscript_sandbox::SandboxConfig` — use
    /// [`VM::with_sandbox_config`] when you already have a full
    /// `hudhudscript_sandbox::SandboxConfig`.
    pub fn with_sandbox(&mut self, config: SandboxConfig) {
        self.sandbox = Some(Box::new(config));
    }

    /// Associated constructor: build a VM pre-configured with the given
    /// [`hudhudscript_sandbox::SandboxConfig`].  Mirrors the interpreter's
    /// `Interpreter::with_sandbox(config) -> Self` so restored tests stay
    /// byte-identical (Kural 1).
    ///
    /// The full sandbox-crate config is flattened into the VM's local
    /// `SandboxConfig` (allowed paths / hosts plus file read/write and
    /// network booleans) because that's what the VM's per-operation
    /// sandbox checks consume.  Process & resource limits from the full
    /// config are honoured at the host layer, not the VM.
    pub fn with_sandbox_config(config: hudhudscript_sandbox::SandboxConfig) -> Self {
        let mut vm = Self::new();
        let allow_file_read = !config.filesystem.allow_read.is_empty();
        let allow_file_write = !config.filesystem.allow_write.is_empty();
        let allow_network = !config.network.allow_domains.is_empty();
        vm.sandbox = Some(Box::new(SandboxConfig {
            allowed_paths: config.filesystem.allow_read.clone(),
            allowed_hosts: config.network.allow_domains.clone(),
            allow_file_read,
            allow_file_write,
            allow_network,
            allow_process: allow_network, // process == network access for MCP
            allowed_commands: vec![],
            denied_commands: vec![
                "rm".into(),
                "dd".into(),
                "mkfs".into(),
                "shutdown".into(),
                "reboot".into(),
            ],
        }));
        vm
    }

    /// Create a VM with all restrictions disabled (for dev/REPL/testing).
    pub fn permissive() -> Self {
        let mut vm = Self::new();
        vm.sandbox = Some(Box::new(SandboxConfig {
            allowed_paths: vec![],
            allowed_hosts: vec![],
            allow_file_read: true,
            allow_file_write: true,
            allow_network: true,
            allow_process: true,
            allowed_commands: vec![],
            denied_commands: vec![],
        }));
        vm
    }

    /// Check that a bytecode Value matches the expected type name (Issue #452).
    pub fn detect_locale(source: &str) -> OutputLocale {
        if source.chars().any(|c| matches!(c, '\u{0600}'..='\u{06FF}')) {
            OutputLocale::Arabic
        } else {
            OutputLocale::Default
        }
    }

    /// Pre-pack all instructions into compact 32-bit form for fast dispatch.
    /// Instructions that cannot be packed (complex payloads) get
    /// Execute bytecode
    pub(crate) fn default() -> Self {
        Self::new()
    }
}
