use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RESOLVER_INVALID_PATH: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(215),
        long_code: "HHS_E_RESOLVER_INVALID_PATH",
        short_code: "E0215",
        title: "Invalid module resolver path",
        short_description: "An import path is malformed — empty, contains illegal characters, or escapes the project root.",
        long_description: "The module resolver translates an import path string into a concrete file location. This error means the path itself is unusable — common cases include empty strings, paths with embedded NUL bytes, paths that traverse above the project root with `../`, or paths using a scheme prefix the resolver does not recognize.

Fix the import statement to use a well-formed path. Use relative imports (`./util`, `../shared/log`) for in-project modules and bare module names (`json`, `http`) for declared dependencies. Avoid absolute filesystem paths in source — they break portability and are rejected by default.

The sandbox refuses paths that escape the project root as a security measure; if you genuinely need to import from outside, configure an additional source root in the manifest rather than using `../../` chains.",
        hints: &["Use `./` and `../` for in-project relative imports", "Use bare names for declared dependencies", "Do not use absolute filesystem paths in import statements", "Configure additional source roots in the manifest if needed"],
        example_bad: Some("import { x } from \"../../../../etc/passwd\""),
        example_good: Some("import { x } from \"./util\""),
        see_also: &["ResolverNotFound", "ModuleLoaderModuleNotFound", "GraphModuleNotFound"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };

pub const RESOLVER_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(216),
        long_code: "HHS_E_RESOLVER_NOT_FOUND",
        short_code: "E0216",
        title: "Resolver could not locate module",
        short_description: "The module resolver applied its search rules and found nothing matching the import path.",
        long_description: "The resolver tried every search root (project src, dependency packages, standard library) and none of them produced a match for this import. This is the resolver-layer counterpart to `ModuleLoaderModuleNotFound`: the resolver gives up before the loader is ever asked to read a file.

Check the spelling and casing of the import. For dependency-rooted imports, make sure the dependency is declared in the manifest and installed (`hhs install`). For relative imports, remember they resolve relative to the importing file. For standard-library imports, check that the name has not been renamed in your version of the standard library.

If you recently added a new source directory, you may need to declare it in the manifest's source-root list — the resolver only looks in known roots.",
        hints: &["Check spelling and casing — names are case-sensitive", "Run `hhs install` to fetch any newly declared dependencies", "Relative imports are relative to the importing file, not cwd", "Declare new source directories in the manifest"],
        example_bad: None,
        example_good: None,
        see_also: &["ResolverInvalidPath", "ModuleLoaderModuleNotFound", "GraphModuleNotFound"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };
