use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const GRAPH_CIRCULAR_DEPENDENCY: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(102),
        long_code: "HHS_E_GRAPH_CIRCULAR_DEPENDENCY",
        short_code: "E0102",
        title: "Circular dependency in module graph",
        short_description: "Two or more modules import each other (directly or transitively), forming a cycle that the loader cannot resolve.",
        long_description: "HudHudScript builds the import graph at parse time and topologically sorts it before evaluation. A cycle (A imports B, B imports A — possibly through C, D, ...) makes that ordering impossible because no module can be fully initialized before its dependencies. The error message lists the offending cycle so you can see exactly which edge to break.

The usual fix is to extract shared types or constants into a third module that both sides import, instead of importing each other. If two modules genuinely need to call into each other at runtime, consider passing one as a parameter (dependency injection) or using actor messages instead of direct imports.

Unlike some languages, HudHudScript does not support 'lazy' or 'forward' imports to paper over cycles — the rule is enforced strictly so that initialization order is always deterministic.",
        hints: &["Read the cycle in the error message: A -> B -> ... -> A", "Extract shared definitions into a third module both sides import", "Replace direct imports with actor messages for runtime cooperation", "Avoid 'utility' modules that import from every part of the codebase"],
        example_bad: Some("// a.hhs
import { foo } from \"./b\"
export fn bar() = foo()
// b.hhs
import { bar } from \"./a\"
export fn foo() = bar()"),
        example_good: Some("// shared.hhs
export fn helper() = 42
// a.hhs
import { helper } from \"./shared\"
// b.hhs
import { helper } from \"./shared\""),
        see_also: &["GraphModuleNotFound", "ModuleLoaderModuleNotFound", "ResolverNotFound"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };

pub const GRAPH_MODULE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(103),
        long_code: "HHS_E_GRAPH_MODULE_NOT_FOUND",
        short_code: "E0103",
        title: "Module missing from dependency graph",
        short_description: "A module referenced by an import edge could not be located in the resolved dependency graph.",
        long_description: "After the resolver builds the module graph, every edge must point to a node that was actually loaded. This error means a node looked up its successor and the graph did not contain it — typically because a module was removed between graph construction and traversal, or because the resolver returned a path the loader could not turn into a node.

This is usually a downstream symptom of an earlier failure: a parse error in a transitive dependency, a resolver path that points outside the project root, or a stale `.hhs-cache` from a previous build. Run `hhs clean` to drop the cache and re-run the load.

If the error persists with a clean cache, it points to a bug in the resolver rather than user code — please file an issue with the import path that triggered it.",
        hints: &["Run `hhs clean` to drop the module graph cache", "Check for earlier parse errors that prevented the module from loading", "Verify the import path resolves to a file inside the project root", "If reproducible after a clean build, file an issue"],
        example_bad: None,
        example_good: None,
        see_also: &["GraphCircularDependency", "ModuleLoaderModuleNotFound", "ResolverInvalidPath"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };
