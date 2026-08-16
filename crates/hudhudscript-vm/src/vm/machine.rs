//! Virtual Machine state structure.

use hudhudscript_async::PromiseRegistry;
use hudhudscript_bytecode::cache_utils::{MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::{
    actor_registry::{ActorMailbox, SharedActorRegistry},
    FunctionChunk, FunctionData, GeneratorState16, SymId, Value16,
};
use hudhudscript_debug::Debugger;
use hudhudscript_governance::Constitution;
use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
use hudhudscript_rag::{SimpleEmbedding, VectorStore};
use hudhudscript_runtime::provider::Provider;
use hudhudscript_tools::ToolRegistry;
use parking_lot::Mutex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::vm::call_state::{VmCallRequest, VmContinuation};
use crate::vm::config_types::{OutputLocale, SandboxConfig};
pub(crate) use crate::vm::machine_types::{CallFrame, ChunkCache};
use crate::vm::registry::ModuleRegistry;

/// Virtual Machine state
pub struct VM {
    /// Feature-gated telemetry counters (GATE 2).
    /// Not present in default builds — zero memory/cpu overhead.
    #[cfg(feature = "telemetry")]
    pub telemetry: crate::vm::telemetry::Telemetry,
    pub gc_heap: Box<hudhudscript_bytecode::gc::GcHeap>,
    /// Operand stack.  Audit v3 Finding 1.1 / S1.1 (PERF-20): stores
    /// Return value relay: function Return/IntAddReturn/IntSubReturn write here.
    /// run_frame_loop reads from here on hit_return path instead of registers[255].
    pub(crate) last_return: Value16,
    /// Global namespace (top-level bindings: functions, constants, classes).
    /// Mirrors the Rune/Lua/Wren pattern: a single HashMap per VM, not a
    /// per-call scope stack. PERF-1 Step 1 migration (2026-04-17) —
    /// All `define_global` / `set_global` / module-level declarations land
    /// here. Inner function/block scopes use `registers` directly.
    pub(crate) globals: FxHashMap<hudhudscript_bytecode::interner::SymbolId, Value16>,
    /// P3: flat Vec for shared top-level globals, indexed by compile-time
    /// slot index.  Hot path LoadGlobal/StoreGlobal reads/writes here
    /// instead of globals HashMap.
    pub(crate) shared_globals_vec: Vec<Value16>,
    /// Upvalue cell sidecar — stack of cell maps, one per active function
    /// call. When a variable is captured by a closure, its value in the
    /// enclosing function is promoted to a shared cell (`Arc<RwLock<Value>>`).
    /// Reads/writes to that name route through the cell instead of the plain
    /// slot entry.  One map is pushed on every function call and popped on
    /// G5-slotvec: (cells, sym_ids) per scope. cells[slot]=None → uninitialized.
    /// sym_ids: Arc shared from chunk (single ref-count bump per call, no clone).
    pub(crate) scope_cells: Vec<(Box<[Option<Arc<parking_lot::RwLock<Value16>>>]>, Arc<[u32]>)>,
    /// Output locale (for number formatting)
    pub(crate) locale: OutputLocale,
    /// F3: Object/dizi eşitlik politikası (varsayılan: Identity)
    pub object_equality: crate::vm::config_types::ObjectEquality,
    /// Try-catch frames: (catch_target_ip, iterator_depth, loop_header_depth)
    pub(crate) try_frames: Vec<(usize, usize, usize)>,
    /// Finally frames: (finally_ip, iter_depth, loop_depth).
    pub(crate) finally_frames: Vec<(usize, usize, usize)>,
    /// Pending abrupt flow recorded by Return / Throw when a finally frame
    /// intercepts them.  The `FinallyExit` instruction consults this at the
    /// end of the finally body: `Some(Return(v))` re-issues the return,
    /// `Some(Throw(v))` re-issues the throw, and `None` falls through to
    /// the after-finally IP.
    ///
    /// Cleared by `FinallyExit` after dispatch so subsequent try/finally
    /// blocks start fresh.  If the finally body itself performs an abrupt
    /// completion (a plain `return` / `throw` inside finally) the pending
    /// flow is *overwritten* — matching the interpreter's
    /// `finally_result.is_err() → return finally_result` semantics.
    pub(crate) pending_flow: Option<crate::vm::types::PendingFlow>,
    /// For-in iterator state: Vec<(elements, variable_name, current_index)>
    pub(crate) iterators: Vec<(Vec<Value16>, String, usize)>,
    /// Parallel to `iterators`: if entry is `Some`, this frame is a generator
    /// and `IterNext` should lazily `advance()` instead of indexing `elements`.
    /// Allows infinite / side-effecting generators to iterate correctly.
    pub(crate) iterator_generators: Vec<Option<Arc<Mutex<GeneratorState16>>>>,
    /// Loop header and exit IP stack for continue/break instructions
    /// Each element is (header_ip, exit_ip)
    pub(crate) loop_headers: Vec<(usize, usize)>,
    /// Class metadata: class_name → (parent, method_names)
    pub(crate) classes: FxHashMap<String, (Option<String>, Vec<String>)>,
    /// Declaration store: kind:name → fields object
    pub(crate) declarations: FxHashMap<String, Value16>,
    /// Effect registry: event_name → handler function chunk name (#351)
    pub(crate) effects: FxHashMap<String, String>,
    /// Relation store: "SubjectA_SubjectB" → relation object (#352)
    pub(crate) relations: FxHashMap<String, Value16>,
    /// Subject template registry: name → SubjectTemplate
    pub(crate) subject_templates: FxHashMap<String, crate::vm::sop_types::SubjectTemplate>,
    /// Event schema registry: event_name → EventSchema
    pub(crate) event_schemas: FxHashMap<String, crate::vm::sop_types::EventSchema>,
    /// Live subject instances: instance_id → SubjectInstance
    pub(crate) subject_instances: FxHashMap<String, crate::vm::sop_types::SubjectInstance>,
    /// SOP0007: Composition rules: "Knight::attack" → Vec<CompositionRule>
    pub(crate) composition_rules: FxHashMap<String, Vec<crate::vm::sop_types::CompositionRule>>,
    /// SOP0009: Field correspondences: "Knight::state::name" → FieldCorrespondence
    pub(crate) field_correspondences: FxHashMap<String, crate::vm::sop_types::FieldCorrespondence>,
    /// ENV0004: Provider defaults from hudhud.toml [providers.NAME]
    pub(crate) toml_providers: FxHashMap<String, FxHashMap<String, String>>,
    /// Full hudhud.toml config as nested object, accessible via config() builtin
    pub(crate) toml_config: Value16,
    /// PROVIDER0002: receiver object during X.call() dispatch
    pub(crate) dispatch_provider_receiver: Option<Value16>,
    /// AGENT0003: agent names for dispatch lookup
    pub(crate) agent_names: FxHashMap<String, ()>,
    /// AGENT0004: swarm names for dispatch lookup
    pub(crate) swarm_names: FxHashMap<String, ()>,
    /// AGENT0005: council names for dispatch lookup
    pub(crate) council_names: FxHashMap<String, ()>,
    /// AGENT0006: community names for dispatch lookup
    pub(crate) community_names: FxHashMap<String, ()>,
    /// Ability handlers: ability_name → chunk_name
    pub(crate) ability_handlers: FxHashMap<String, String>,
    /// Active constitutions (typed governance objects from hudhudscript-governance)
    pub(crate) constitutions: FxHashMap<String, Constitution>,
    /// Active constitution name
    pub(crate) active_constitution: Option<String>,
    /// Provider / LLM provider for agent calls (Kural 7 — shared dispatch).
    ///
    /// Populated by `VM::set_provider` from the runtime harness or a
    /// `register_provider(...)` builtin. The shared provider dispatcher
    /// reads this through `ProviderContext::provider_get_provider`.
    pub(crate) provider: Option<Arc<dyn Provider>>,
    /// Named provider registry (interpreter-parity API, Kural 2).
    ///
    /// When a script sets `provider = "name"` in scope and calls
    /// `this.call({...})`, `provider_get_provider` reads the variable
    /// and resolves the name through this registry.  Populated via
    /// `VM::set_provider_registry` from runtime harnesses or tests.
    pub(crate) provider_registry: Option<Arc<hudhudscript_runtime::provider::ProviderRegistry>>,
    /// Tool registry for agent/script tool dispatch (Issue #23 parity).
    ///
    /// Populated via `VM::set_tool_registry`; mirrors the interpreter's
    /// per-runtime registry so hand-rolled harnesses can hand the same
    /// `Arc<ToolRegistry>` to either executor without branching.
    pub(crate) tool_registry: Option<Arc<ToolRegistry>>,
    /// MCP tool definitions discovered via `tools/list` (Issue #436).
    ///
    /// Keyed by server name; consulted at dispatch time to validate
    /// `mcp.Server.toolName()` calls.  Shared `Arc<Mutex<_>>` so the
    /// `&self`-only `McpContext` trait impl can still read it.
    pub(crate) mcp_tool_definitions: Arc<Mutex<HashMap<String, Vec<McpToolDefinition>>>>,
    /// Registered MCP clients keyed by server name.
    ///
    /// Populated by `VM::register_mcp_client` (typically from the runtime
    /// harness). Consumed by the shared MCP dispatcher through
    /// `McpContext::mcp_get_client`. Wrapped in `Arc<Mutex<_>>` so the
    /// `&self`-only `McpContext` trait impl can still clone out an
    /// `Arc<McpClient>` on demand.
    pub(crate) mcp_clients: Arc<Mutex<HashMap<String, Arc<McpClient>>>>,
    /// G08: circular module import guard state (shared with sub-VMs).
    pub(crate) module_load_context:
        Arc<parking_lot::Mutex<crate::vm::module_load_context::ModuleLoadContext>>,
    /// Call depth tracking for the recursion limit (Issue #965). Frames
    /// push/pop this on the trampoline; hitting `max_call_depth` returns
    /// a graceful error instead of exhausting the OS stack.
    pub(crate) call_depth: usize,
    /// Maximum allowed call depth. Default:
    /// [`hudhudscript_errors::constants::MAX_CALL_DEPTH`] (2000).
    /// Configurable via `VM::with_max_call_depth`.
    /// Hard ceiling configurable via `VM::with_max_call_depth_ceiling`.
    pub(crate) max_call_depth: usize,
    pub(crate) max_call_depth_hard_ceiling: usize,
    /// Fuel/gas limit for execution
    pub(crate) fuel_limit: Option<u64>,
    /// Remaining fuel
    pub(crate) fuel_remaining: u64,
    /// Register arena initial size in bytes (default: 64KB).
    pub(crate) register_arena_size: usize,
    /// Actor mailbox capacity (default: 128).
    pub(crate) mailbox_capacity: usize,
    /// Max MCP servers (default: 128).
    pub(crate) max_mcp_servers: usize,
    /// Builtin iteration limit (default: 10_000).
    pub(crate) max_builtin_iter: usize,
    /// Non-Linux default stack size in bytes for remaining_stack_bytes().
    pub(crate) default_stack_bytes: usize,
    /// Default provider timeout in seconds (Issue #446).
    pub(crate) provider_timeout_secs: u64,
    /// Sandbox config: allowed file paths and network hosts.
    /// PERF-50: `Box` the config so the `Option<...>` niche-optimises to a
    /// single nullable pointer.  `SandboxConfig` holds three `Vec`s and three
    /// `bool`s (~72 B); inlining as `Option<SandboxConfig>` inflated the VM
    /// struct by ~80 B per instance.  The sandbox is only consulted on
    /// file/network/MCP ops, so indirect access is irrelevant to the hot
    /// dispatch loop.
    /// G12 (exp/unboxed-float): sıcak döngünün float-kanıtlı yerelleri —
    /// tag'siz f64 slot dosyası. F* komutları yalnız burayı okur/yazar;
    /// döngü sınırlarında FLoadNum/FStoreNum köprüler.
    pub(crate) f_slots: [f64; 64],
    pub(crate) sandbox: Option<Box<SandboxConfig>>,
    /// M2: `[runtime] allow_insecure_http` — SSE MCP için http:// + loopback
    /// SSRF muafiyeti opt-in'i. `allow_network`'ten kasıtlı olarak AYRI izin
    /// (biri ağ erişimi, diğeri şifresiz http). Default: false.
    pub(crate) allow_insecure_http: bool,
    /// HOST-3: runtime host-access policy. Default is permissive to preserve
    /// existing CLI behaviour when no `[host_access]` section is configured.
    pub(crate) host_access_policy: crate::vm::host_access::HostAccessPolicy,
    /// STM: shared TVar registry (Kural 7 — `hudhudscript-stm`).
    /// Maps script-visible string handles to live `Arc<TVar<Value>>`.
    pub(crate) tvars: hudhudscript_stm::TVarRegistry<Value16>,
    /// Class context stack for proper super call resolution in nested contexts
    pub(crate) class_context_stack: Vec<hudhudscript_bytecode::SymId>,
    /// Scratch slot used by `call_method_on_value` for `Value::Instance`
    /// receivers: the post-method updated `this` is stashed here before
    /// the method's frame-local `this` is restored, so the MethodCall
    /// instruction can write the mutation back to the original receiver
    /// variable (fixes the setter/counter-style parity gap).
    /// PERF-50: `Box` the `Value` so `Option<...>` niche-optimises to a
    /// single nullable pointer.  `Value` is a 32-byte enum; inlining this
    /// `Option<Value>` cost ~40 B per VM instance even when unset.  Every
    /// call goes through `take()` / `Some(...)` which is a single pointer
    /// write either way.
    pub(crate) last_instance_mutation: Option<Box<Value16>>,
    /// Set of variable names declared as `const` (immutable after initial assignment)
    pub(crate) immutables: HashSet<hudhudscript_bytecode::interner::SymbolId>,
    /// Avoids 30M+ interner::resolve calls in hot loops.
    pub(crate) sym_cache: FxHashMap<u32, String>,
    /// in get_var_cloned hot path (IssUE-007). Uses RefCell for interior mutability.
    pub(crate) name_sym_cache: std::cell::RefCell<FxHashMap<String, u32>>,
    pub(crate) length_sym_id: u32,
    pub(crate) push_sym_id: u32,
    pub(crate) math_obj: Option<Value16>,
    pub(crate) json_obj: Option<Value16>,

    /// Whether we are inside an `atomically()` block (#518)
    pub(crate) in_stm_context: bool,
    /// Active STM transaction — `Some` only inside `atomically(fn)`.
    /// All `tvar_read` / `tvar_write` during this window go through the shared
    /// `Transaction` so conflicts and versioning are detected exactly like
    /// the interpreter.
    /// PERF-50: `Box` the transaction so `Option<...>` niche-optimises to a
    /// single nullable pointer.  `Transaction<Value>` holds two `HashMap`s
    /// (~96 B); outside an `atomically(fn)` block this slot is permanently
    /// `None`, so the box is never allocated on the hot path.
    pub(crate) current_tx: Option<Box<hudhudscript_stm::Transaction<Value16>>>,
    /// Issue #661: Debugger for breakpoints, stepping, and DAP integration
    /// PERF-39: `Box` the debugger so the `Option<...>` niche-optimises to
    /// a single nullable pointer.  Every VM instruction does
    /// `if self.debugger.is_some() { ... }`; with an inline Option<Debugger>
    /// that's a ≥200-byte discriminant check per instruction.  With the
    /// boxed form it's a null-pointer compare.
    pub(crate) debugger: Option<Box<Debugger>>,
    /// Issue #661: Current source file name (for debugger location tracking)
    pub(crate) current_file: Option<String>,
    /// #702 / Kural 7: Shared actor registry (hudhudscript-shared-builtins)
    ///
    /// Holds supervised actor refs + lifecycle metadata. Both runtimes run
    /// the *same* registry implementation now; the VM no longer ships its
    /// own parallel mailbox tables. `actor_mailboxes` below owns the
    /// receive-side of every spawn call, keyed by the registry-generated
    /// id, so bytecode `Receive` instructions can drain a specific actor's
    /// inbox without contending for the registry lock.
    pub(crate) actors: Arc<SharedActorRegistry<Value16>>,
    /// Receive-side of each actor owned by this VM (id → ActorMailbox).
    pub(crate) actor_mailboxes: HashMap<String, ActorMailbox<Value16>>,
    /// Shared thread-backed promise registry (Kural 7 / #726).
    ///
    /// Replaces the previous pair of ad-hoc `promise_receivers` and
    /// `promise_results` HashMaps. The registry consolidates the VM's
    /// side of the async pipeline into a single type that lives in
    /// `hudhudscript-async` and can be reused by any future non-tokio
    /// runtime. The Await instruction drives resolution exclusively
    /// through this registry — no ad-hoc pending-to-null fallback.
    pub(crate) promise_registry: PromiseRegistry<Value16>,
    /// #728/#892: RAG embedding provider for cosine similarity recall
    pub(crate) rag_embedder: SimpleEmbedding,
    /// #892: RAG vector stores delegated to hudhudscript_rag::VectorStore (per store_name)
    pub(crate) rag_stores: FxHashMap<String, VectorStore>,
    /// #892: Reverse mapping: store_name → (text → [entry_uuid]) for forget-by-content
    pub(crate) rag_text_to_ids: HashMap<String, HashMap<String, Vec<String>>>,
    /// Register file for register-based VM instructions.
    /// Boxed so call/return can swap the entire bank via
    /// std::mem::swap (O(1) pointer exchange) instead of
    /// copying 256 Value16s (AÇIK-8 / PERF-REGSWAP).
    pub(crate) registers: crate::vm::register_arena::RegisterArena,
    /// Call-stack parallel to `call_stack_names`: each entry holds the
    /// Used by `get_var_cloned` / `upvalue_cell_for` for slot-based lookups.
    /// PERF-T2-3: raw pointer eliminates Arc::clone atomic refcount ops.
    /// generator), all valid for the VM lifetime.
    pub(crate) call_stack_local_syms: Vec<*const Vec<(u32, usize, Option<usize>)>>,
    /// Owned local_syms allocations that must be freed on VM drop.
    /// Entries correspond to call_stack_local_syms slots created via
    /// included here because ChunkCache owns them.
    pub(crate) owned_local_sym_refs: Vec<*const Vec<(u32, usize, Option<usize>)>>,
    /// Pending call request set by execute_instructions on StepAction::Call
    pub(crate) pending_call: Option<(SymId, u32, u8, u8, u8, usize)>, // (func_sym, function_idx, arg_count, first_arg, dst, ip)
    /// Heap-owned VM-to-VM request consumed only by the outer frame driver.
    pub(crate) pending_vm_call: Option<Box<VmCallRequest>>,
    /// Large multi-stage call state. `StepAction` carries only a marker/id.
    pub(crate) vm_continuations: Vec<VmContinuation>,
    /// Flag: the next exec_call in the trampoline should set class_context on the pushed frame.
    pub(crate) pending_super_call: bool,
    /// T3-1-B: Trampoline call-frame stack.
    pub(crate) frame_stack: Vec<CallFrame>,

    /// PERF0004: base index into the operand stack for the current call frame.
    /// Same flat-Vec trick as local_slot_base. On call: save base, set new
    /// base = stack.len(). On return: pop return value, truncate to base,
    /// restore base.
    pub(crate) stack_frame_base: usize,
    /// Audit v3 Finding 2.1 (PERF-17): store interned ids instead of owned
    /// Strings.  Every function call used to allocate a `String::from(name)`
    /// via `call_stack_names.push(name.to_string())`; now it's a 4-byte
    /// `SymId` and resolve happens lazily when formatting stack traces.
    pub(crate) call_stack_names: Vec<hudhudscript_bytecode::SymId>,
    /// #928: External module handler registry
    pub(crate) module_registry: ModuleRegistry,
    /// Optional module resolver for LoadModule instruction (#921)
    pub(crate) module_resolver: Option<Box<dyn hudhudscript_errors::ModuleResolver>>,
    /// Optional execution deadline for time-based yielding
    pub(crate) execution_deadline: Option<std::time::Instant>,
    /// External cancellation token — checked every N instructions
    pub(crate) cancellation_token: Arc<std::sync::atomic::AtomicBool>,
    /// IP where execution was suspended (for `execute_resume`)
    pub(crate) suspended_ip: Option<usize>,
    /// CROSS-1 (TCO): scratch slot for tail-call argument drain.
    /// `step_one`'s `Instruction::TailCall` arm drains args here, then
    /// signals `StepAction::TailCall` (a zero-size marker) so the
    /// outer loop can consume the Vec without the enum having to
    /// carry it.  Keeping the Vec off the `StepAction` enum keeps
    /// `Result<StepAction, CompileError>` tiny on the hot dispatch
    /// path (Vec<Value> is 24 B and would inflate every step's
    /// return value).  Reused across tail calls via `take()` in the
    /// handler — no per-call allocation once grown.
    pub(crate) tco_args: Option<Vec<Value16>>,
    /// AÇIK-9: Reusable scratch buffer for argument lists > 8.
    /// Eliminates per-call Vec allocation by clearing + reusing
    /// the same backing storage across calls.
    pub(crate) args_scratch: Vec<Value16>,
    /// Lazy generator support: sender for yielded values.
    /// When set, the `Yield` instruction sends through this channel (blocking
    /// on the rendezvous channel until `next()` consumes it) instead of
    /// V2-B: Yield-tabanlı generator'ların DetachedGraph receiver'ları.
    /// Key = state.yield_id; receiver heap'ten bağımsız → GC trace gerekmez.
    pub(crate) yield_receivers:
        FxHashMap<u64, std::sync::mpsc::Receiver<hudhudscript_bytecode::gc_detach::DetachedGraph>>,
    /// H-BLOCKING: Detached promise receivers (detach in thread, attach on await).
    pub(crate) detached_promises: FxHashMap<
        String,
        std::sync::mpsc::Receiver<Result<hudhudscript_bytecode::gc_detach::DetachedGraph, String>>,
    >,
    pub(crate) next_yield_id: u64,
    // Eski yield_sender (Value16) — V2-B'de DetachedGraph'e geçildi. Hâlâ kurulumda kullanılır
    // ama kanal artık gc_detach::DetachedGraph taşır.
    pub(crate) yield_sender:
        Option<std::sync::mpsc::SyncSender<hudhudscript_bytecode::gc_detach::DetachedGraph>>,
    /// PERF0011: Unified chunk cache — single FxHashMap lookup for both
    /// packed instructions and local symbol metadata (same key for both).
    pub(crate) chunk_cache: FxHashMap<usize, Arc<ChunkCache>>,
    /// Inline cache for chunk_cache — skip FxHashMap on repeated calls.
    pub(crate) chunk_cache_last_key: usize,
    pub(crate) chunk_cache_last_val: Option<Arc<ChunkCache>>,
    /// Current chunk pointer set by the trampoline before executing instructions.
    /// Used by property/call IC fast-paths to access ic_slots.
    pub(crate) current_chunk_ptr: *const FunctionChunk,
    /// PERF-2: compile-time SymbolId → top-level slot mapping.
    /// P3: stored as a flat Vec indexed by SymbolId to avoid HashMap
    /// probe cost on every LoadGlobal/StoreGlobal/DeclGlobal.
    /// GÖREV 4d: char decomposition cache for non-ASCII string indexing.
    /// Keyed by string content (avoid repeated chars() iteration).
    pub(crate) str_chars_cache: FxHashMap<String, Vec<char>>,
    /// `u32::MAX` means "not a top-level local"; low byte = slot,
    /// bit 8 = shared flag.
    pub(crate) main_local_slots: Vec<u32>,
    /// P5.1: GC-root olarak izlenecek sabit havuzu değerleri (Bytecode +
    /// FunctionChunk constants). execute() girişinde + chunk cache eklemede doldurulur.
    pub(crate) gc_constant_roots: Vec<Value16>,
    /// G3-2: Already registered chunk roots — guards duplicate extend.
    pub(crate) constant_root_chunks: FxHashSet<*const hudhudscript_bytecode::FunctionChunk>,
    /// Callee cache indexed by function-name sym_id.
    /// `(*const FunctionChunk, *const Vec<String> params)`.
    /// Arc::clone atomic refcount ops on every recursive call.
    /// Bytecode::functions (Arc), their constants are in gc_constant_roots
    /// (add_chunk_constants, run.rs:11), and cache is drained on VM drop.
    pub(crate) call_cache: Vec<
        Option<(
            *const FunctionData,
            *const FunctionChunk,
            *const Vec<String>,
        )>,
    >,

    pub(crate) ability_cache: [(
        Option<(
            String,
            hudhudscript_bytecode::SymId,
            Arc<hudhudscript_bytecode::FunctionChunk>,
            String,
        )>,
        u8,
    ); 2],

    /// P3: pre-interned SymId for hot-path set_var/get_var in class method dispatch.
    pub(crate) this_sym: u32,
    pub(crate) cur_this: Value16,
}
