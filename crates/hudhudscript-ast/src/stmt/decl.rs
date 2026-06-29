use crate::stmt::Decorator;
use crate::{Expr, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decl {
    /// Agent declaration: agent MyAgent { ... }
    Agent {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Agent action: executable action inside an agent
    /// action add(a, b) { return a + b; }
    /// action add_async(a, b) async { return a + b; }
    AgentAction {
        agent_name: String,
        name: String,
        params: Vec<String>,
        body: Vec<super::Stmt>,
        is_async: bool,
        span: Span,
    },

    /// SOP ability: on attack(self, target) { body }
    /// or subject-scoped: on Knight.attack(self, target) { body }
    Ability {
        name: String,
        /// Optional subject type for scoped capabilities (e.g. "Knight" in `on Knight.attack`)
        subject_type: Option<String>,
        params: Vec<String>,
        body: Vec<super::Stmt>,
        span: Span,
    },

    /// Action declaration: action myAction { ... }
    Action {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Tool declaration: tool myTool { ... }
    Tool {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Resource declaration: resource myResource { ... }
    Resource {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// HudHudScript `use`-style import: use postgres as db;
    ///
    /// Distinct from `Stmt::Import { .. }`, which represents ES-module style imports.
    Import {
        module: String,
        alias: Option<String>,
        span: Span,
    },

    /// Constitution declaration: constitution MyConstitution { ... }
    Constitution {
        name: String,
        description: Option<String>,
        laws: Vec<LawDecl>,
        span: Span,
    },

    /// Law declaration: law MyLaw { ... }
    /// Issue #474: rules stores parsed Expr for compile-time validation.
    Law {
        name: String,
        description: String,
        enforcement_level: String, // "mandatory", "advisory", "optional"
        rules: Vec<Expr>,
        span: Span,
    },

    /// Council declaration: council MyCouncil { ... }
    Council {
        name: String,
        constitution: String, // Constitution ID
        members: Vec<CouncilMemberDecl>,
        rules: Vec<String>, // Rule IDs
        span: Span,
    },

    /// Rule declaration: rule MyRule: { ... }
    Rule {
        name: String,
        conditions: Vec<ConditionDecl>,
        actions: Vec<ActionDecl>,
        priority: u32,
        span: Span,
    },

    /// Swarm declaration: swarm MySwarm { ... }
    Swarm {
        name: String,
        agents: Vec<String>, // Agent IDs
        strategy: String,    // "parallel", "sequential", "competitive", "collaborative"
        span: Span,
    },

    /// Community declaration: community MyCommunity: { ... }
    Community {
        name: String,
        members: Vec<String>,  // Agent IDs
        councils: Vec<String>, // Council IDs
        culture: CultureDecl,
        span: Span,
    },

    /// Provider declaration: provider openai_gpt4 { ... }
    Provider {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Protocol declaration: protocol MyProtocol: { execution: parallel, governance: MyGov, ... }
    Protocol {
        name: String,
        /// Execution type: "parallel" | "sequential" | "competitive" | "roundRobin"
        execution: Option<String>,
        /// Governance model reference
        governance: Option<String>,
        /// Timeout in seconds
        timeout: Option<f64>,
        /// Session lifecycle hooks: onStart, onMemberStart, onMemberComplete, onComplete, onError
        session: Vec<(String, Expr)>,
        span: Span,
    },

    /// Governance declaration: governance MyGov: democracy { ... }
    Governance {
        name: String,
        /// Base governance type: "democracy", "monarchy", "republic", etc.
        base_type: String,
        /// Override fields: constitution_compliance, law_flexibility, rule_enforcement, description
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Subject declaration (SOP): @ai subject Player has Fighter { state health: 100, can attack, ... }
    Subject {
        name: String,
        /// Decorators: @ai, @payment, @cloud, @hudhud, @custom(...)
        decorators: Vec<Decorator>,
        /// Harrison & Ossher: which base subject this view extends (via `of` keyword)
        of_subject: Option<String>,
        /// Roles this subject has (via `has` keyword)
        roles: Vec<String>,
        /// State declarations: (name, initial_value)
        states: Vec<(String, Expr)>,
        /// Capabilities: what the subject can do (string names from `can` keyword)
        capabilities: Vec<String>,
        /// Ability definitions inside subject body: on attack(self, target) { ... }
        ability_defs: Vec<SubjectAbilityDef>,
        /// Intents: what the subject can intend
        intents: Vec<String>,
        /// Uses declarations: (provider_name, optional via channel)
        uses: Vec<(String, Option<String>)>,
        /// Memory fields: (name, value)
        memory: Vec<(String, Expr)>,
        /// Perception fields: (name, value)
        perception: Vec<(String, Expr)>,
        /// Additional fields: (key, value)
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Relation declaration (SOP): relation Player <-> Merchant { trust: 50 }
    Relation {
        subject_a: String,
        subject_b: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Effect handler (SOP): effect on Damage(target, amount) { body }
    Effect {
        event_name: String,
        params: Vec<String>,
        body: Vec<super::Stmt>,
        span: Span,
    },

    /// Role declaration (SOP): role Fighter { can attack, can defend }
    Role {
        name: String,
        /// Capabilities this role grants (from `can` keyword)
        capabilities: Vec<String>,
        /// Additional key-value fields
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// SOP0007: Composition rules for Harrison & Ossher SOP
    Compose {
        base_subject: String,
        rules: Vec<ComposeRule>,
        /// SOP0009: field correspondence rules
        field_rules: Vec<(String, FieldCorrespondence)>,
        span: Span,
    },

    /// Store declaration (RAG): store my_store { backend: "hnsw", dimensions: 1536 }
    Store {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Strategy declaration: strategy MyStrategy: { execution: parallel, governance: MyGov, ... }
    Strategy {
        name: String,
        /// Execution type: "parallel" | "sequential" | "competitive" | "roundRobin"
        execution: Option<String>,
        /// Governance model reference
        governance: Option<String>,
        /// Timeout in seconds
        timeout: Option<f64>,
        /// Required permissions for this strategy
        permissions: Vec<String>,
        /// Realm this strategy operates in
        realm: Option<String>,
        /// Session lifecycle hooks
        session: Vec<(String, Expr)>,
        span: Span,
    },

    // ── Issue #285: dedicated domain-specific AST variants ────────────────
    /// Entity declaration: entity Player { data health: Number = 100, ... }
    Entity {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// State machine declaration: statemachine TrafficLight { state Red { on event(Timer) -> Green } ... }
    StateMachine {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Event declaration: event Damage { amount: Number, source: String }
    Event {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Contract declaration: contract Trade: { parties: ["A", "B"], ... }
    Contract {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Treaty declaration: treaty Peace: { signatories: ["X", "Y"], ... }
    Treaty {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    // ── Issue #278: Music DSL declarations ────────────────────────────────
    /// Music declaration: note/chord/melody/harmony/rhythm/tempo/scale
    Music {
        kind: String,
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    // ── Issue #536: UI declarations ─────────────────────────────────────
    /// UI app declaration: ui MyApp { entry: Main, screen Main { ... }, component Card { ... } }
    UiApp {
        name: String,
        entry_screen: Option<String>,
        screens: Vec<UiScreenDecl>,
        components: Vec<UiComponentDecl>,
        span: Span,
    },

    // ── Issue #537: Deploy declarations ─────────────────────────────────
    /// Deploy declaration: deploy MyApp { target { web { ... } }, github { ... } }
    Deploy {
        name: String,
        targets: Vec<DeployTargetDecl>,
        providers: Vec<DeployProviderDecl>,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    // ── Loop engineering (LP-001) ──
    Loop {
        name: String,
        mode: RunModeAst,
        items: Vec<LoopItemAst>,
        goal: Option<GoalSpecAst>,
        span: Span,
    },
    RunLoop {
        name: String,
        span: Span,
    },
    RunChain {
        name: String,
        span: Span,
    },
    Step {
        name: String,
        params: Vec<String>,
        body: Vec<super::Stmt>,
        gate: Option<StepGateAst>,
        span: Span,
    },
    Gate {
        name: String,
        branches: Vec<GateBranchAst>,
        else_target: GateTargetAst,
        span: Span,
    },
    Chain {
        name: String,
        mode: RunModeAst,
        links: Vec<ChainLinkAst>,
        span: Span,
    },
    AttachStep {
        targets: Vec<AttachStepTarget>,
        loop_name: String,
        span: Span,
    },
    AttachLoop {
        loop_name: String,
        chain_name: String,
        mode: Option<RunModeAst>,
        on_done: Option<ChainTargetAst>,
        on_fail: Option<ChainTargetAst>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainLinkAst {
    pub loop_name: String,
    pub inline_loop: Option<Box<Decl>>,
    pub on_done: ChainTargetAst,
    pub on_fail: ChainTargetAst,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainTargetAst {
    Next,
    ChainDone,
    ChainFail,
    Loop(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachStepTarget {
    pub step: String,
    pub gate: Option<String>,
}

/// B: Goal specification for agentic loops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalSpecAst {
    pub metric: String,
    pub target: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunModeAst { Once, Times(u64), Cyclic, UntilConverged, Until(Expr) }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopItemAst {
    InlineStep(Box<Decl>),
    UseStep { name: String, alias: Option<String>, args: Vec<Expr> },
    AttachGate { gate: String, step: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepGateAst {
    pub name: String,
    pub branches: Vec<GateBranchAst>,
    pub else_target: GateTargetAst,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateBranchAst {
    pub cond: Expr,
    pub target: GateTargetAst,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateTargetAst {
    Step(String),
    Loop(String),
    LoopStep(String, String),
    Done,
    Fail,
    Continue,
    Retry,
    Pause,
    Approval,
    Escalate,
}

// ── Issue #536: UI AST helper types ─────────────────────────────────────

/// Screen declaration: screen Main(params) { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiScreenDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<UiNode>,
    pub span: Span,
}

/// Component declaration: component Card { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDecl {
    pub name: String,
    pub props: Vec<(String, Option<Expr>)>,
    pub body: Vec<UiNode>,
    pub span: Span,
}

/// A node inside a UI body — widget, var, event, or platform block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiNode {
    /// Widget: Button "Click" { color: "red" } { children... }
    Widget {
        widget_type: String,
        label: Option<Expr>,
        props: Vec<(String, Expr)>,
        events: Vec<(String, Expr)>,
        children: Vec<UiNode>,
        style: Vec<(String, Expr)>,
        span: Span,
    },
    /// Local state: var count = 0
    Var {
        name: String,
        value: Expr,
        span: Span,
    },
    /// Event handler: on_click: handler
    Event {
        event_name: String,
        handler: Expr,
        span: Span,
    },
    /// Platform-specific block: platform:desktop { ... }
    PlatformBlock {
        platform: String,
        body: Vec<UiNode>,
        span: Span,
    },
    /// Inline expression (e.g. function call, conditional)
    Expr(Expr),
}

// ── Issue #537: Deploy AST helper types ─────────────────────────────────

/// Deploy target: web { framework: "nextjs", ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployTargetDecl {
    pub platform: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

/// Deploy provider: github { on: "push", branch: "main", ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployProviderDecl {
    pub name: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

/// Law declaration within a constitution
///
/// Issue #474: `rules` stores parsed `Expr` nodes instead of raw strings,
/// enabling compile-time validation and proper evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LawDecl {
    pub name: String,
    pub description: String,
    pub enforcement_level: String,
    pub rules: Vec<Expr>,
    pub span: Span,
}

/// Council member declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouncilMemberDecl {
    pub agent_id: String,
    pub role: String,
    pub span: Span,
}

/// Condition declaration for rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionDecl {
    pub condition_type: String, // "equals", "not_equals", "greater_than", etc.
    pub field: String,
    pub value: Expr,
    pub span: Span,
}

/// Action declaration for rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDecl {
    pub action_type: String, // "allow", "deny", "require", "execute", "notify"
    pub params: Vec<(String, Expr)>,
    pub span: Span,
}

/// Culture declaration for communities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultureDecl {
    pub values: Vec<String>,
    pub norms: Vec<String>,
    pub communication_style: String, // "formal", "informal", "technical", "collaborative"
    pub span: Span,
}

/// Agent action declaration inside an agent body
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentActionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<super::Stmt>,
    pub is_async: bool,
    pub span: Span,
}

/// SOP ability definition inside a subject body: on attack(self, target) { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAbilityDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<super::Stmt>,
    pub span: Span,
}

/// SOP0007: Composition mode for Harrison & Ossher composition rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComposeMode {
    Combine(Vec<String>),
    Override(String),
    Before(String),
    After(String),
}

/// SOP0007: A single composition rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposeRule {
    pub ability_name: String,
    pub mode: ComposeMode,
}

/// SOP0009: Field correspondence mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldCorrespondence {
    Correspond,
    Separate,
}
