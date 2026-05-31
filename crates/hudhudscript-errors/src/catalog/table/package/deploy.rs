use super::*;
use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const PACKAGE_BUILD_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(167),
        long_code: "HHS_E_PACKAGE_BUILD_FAILED",
        short_code: "E0167",
        title: "Package build step failed",
        short_description: "The package's build script returned a non-zero exit status, so the build cannot be considered successful.",
        long_description: "The package manager invoked the package's build script (compilation, code generation, native shim build) and the script exited with an error. The wrapped output is the build log; the actual failure is whatever the build tool reported.

Read the log from the bottom up — the first failure is usually the root cause and everything below it is consequence. Common causes are missing system dependencies (a C compiler, `pkg-config`, a `-dev` package), an incompatible toolchain version, or a transient out-of-memory condition during compilation.

Reproduce the failure outside the package manager by running the build script directly in the package's source directory; that gives you a faster edit-compile-test loop and full control over the environment.",
        hints: &["Read the build log from the bottom up to find the root cause", "Install missing system dependencies (cc, pkg-config, -dev packages)", "Reproduce by running the build script directly in the source dir", "Check that your toolchain version matches the package's requirements"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageDependencyResolution", "PackageIo", "NativeBuildError"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_CHECKSUM_MISMATCH: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(168),
        long_code: "HHS_E_PACKAGE_CHECKSUM_MISMATCH",
        short_code: "E0168",
        title: "Downloaded package failed checksum verification",
        short_description: "The downloaded archive's hash does not match the checksum recorded in the lockfile or registry index.",
        long_description: "Every package download is verified against an expected hash from the lockfile or registry index. The download succeeded but produced bytes that hash to a different value. This is a security guarantee, not a soft warning — the package manager refuses to install content it cannot authenticate.

Most commonly the cause is a transient mirror inconsistency or a man-in-the-middle proxy that altered the bytes. Re-run the install — a second attempt against a different mirror often succeeds. If the failure is reproducible, the registry entry may have been re-published over the same version (which is itself a policy violation); contact the maintainer.

Never bypass this check. The expected hash is the entire point of the lockfile, and a mismatch means you would be running code that nobody (including the original publisher) has signed off on.",
        hints: &["Re-run the install — transient mirror issues usually self-heal", "Check for a corporate proxy or middlebox tampering with downloads", "Never bypass checksum verification — it is a security boundary", "If reproducible, contact the package maintainer about a re-publish"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageNetwork", "PackageSecurityVulnerability", "PackagePackageNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_DEPENDENCY_RESOLUTION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(169),
        long_code: "HHS_E_PACKAGE_DEPENDENCY_RESOLUTION",
        short_code: "E0169",
        title: "Dependency resolver could not satisfy constraints",
        short_description: "The resolver found no combination of versions that satisfies every dependency's semver constraints simultaneously.",
        long_description: "HudHudScript's package resolver is SAT-based: it explores the graph of dependency versions trying to find an assignment that satisfies every constraint. This error means no such assignment exists — typically because two of your dependencies require incompatible versions of a common transitive dependency (the classic 'diamond conflict').

The error message lists the conflicting requirements. Possible fixes: relax one of your direct dependencies' version constraints, upgrade a dependency to a release that aligns its transitive requirements with the other side, or temporarily add a `[patch]` override that pins the shared dependency to a specific version both sides will accept.

If the conflict involves a peer dependency, the solution is usually to upgrade both sides together rather than holding one back.",
        hints: &["Read the error to find the two conflicting requirements", "Run `hhs tree` to visualize the dependency graph", "Try `hhs update <pkg>` to pull in newer compatible releases", "Use a `[patch]` section to override a problematic transitive dep"],
        example_bad: Some("[dependencies]
foo = \"1.0\"  # needs bar ^1.0
baz = \"2.0\"  # needs bar ^2.0"),
        example_good: Some("[dependencies]
foo = \"1.5\"  # release that updated to bar ^2.0
baz = \"2.0\""),
        see_also: &["PackageVersionNotFound", "PackagePackageNotFound", "PackageInvalidVersion"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_ENTRY_POINT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(170),
        long_code: "HHS_E_PACKAGE_ENTRY_POINT_NOT_FOUND",
        short_code: "E0170",
        title: "Package entry point file missing",
        short_description: "The file declared as the package's entry point in its manifest does not exist on disk.",
        long_description: "Every package manifest declares an entry point (the module the package starts from). The package manager could not find that file. Either the manifest's `main`/`entry` field points at the wrong path, or the file was deleted or never committed.

Open the package manifest, locate the entry-point field, and confirm the path is correct relative to the package root. Check that the file is committed to source control if you are installing from a Git source, or included in the published archive if you are installing from a registry.

For library packages with no executable entry point, make sure the manifest uses the library form rather than the application form — they accept different fields.",
        hints: &["Check the `main` / `entry` field in the package manifest", "Verify the file is committed (for git deps) or packed (for registry deps)", "Use a path relative to the package root, not the workspace root", "Library packages should use the library manifest form"],
        example_bad: Some("[package]
name = \"foo\"
main = \"src/main.hhs\"  # file does not exist"),
        example_good: Some("[package]
name = \"foo\"
main = \"src/lib.hhs\""),
        see_also: &["PackageInvalidPackageName", "ModuleLoaderModuleNotFound", "PackageToml"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_INVALID_PACKAGE_NAME: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(171),
        long_code: "HHS_E_PACKAGE_INVALID_PACKAGE_NAME",
        short_code: "E0171",
        title: "Invalid package name",
        short_description: "The package name violates the naming rules (length, allowed characters, reserved prefixes).",
        long_description: "HudHudScript package names must be lowercase, start with a letter, contain only ASCII letters, digits, hyphens, and underscores, and stay within length limits. They cannot collide with reserved names (`std`, `core`, `hhs`, ...) or use restricted prefixes. The name you provided breaks one of these rules.

Rename the package in its manifest. If you are publishing, pick a name that is also unique within the registry — a duplicate would be rejected at upload anyway. Avoid hyphen/underscore inconsistencies (`my-pkg` vs `my_pkg`) since they hash differently and confuse users.

If you depend on a third-party package with an unusual name and the resolver rejects it, the issue is likely a registry-side typo; report it to the maintainer rather than working around it locally.",
        hints: &["Use lowercase letters, digits, `-`, and `_` only", "Start with a letter, not a digit or punctuation", "Avoid reserved names: `std`, `core`, `hhs`", "Pick one of `-` or `_` and stick with it"],
        example_bad: Some("[package]
name = \"My_Cool_Pkg!\"  # uppercase + invalid char"),
        example_good: Some("[package]
name = \"my-cool-pkg\""),
        see_also: &["PackageInvalidVersion", "PackageToml", "PackagePackageNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_INVALID_VERSION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(172),
        long_code: "HHS_E_PACKAGE_INVALID_VERSION",
        short_code: "E0172",
        title: "Invalid semver version string",
        short_description: "A version string in the manifest or lockfile is not a valid semantic version.",
        long_description: "HudHudScript packages use strict semantic versioning (`MAJOR.MINOR.PATCH` plus optional pre-release and build metadata). The version string the package manager read does not parse as semver — common mistakes are `1.0` (missing patch), `v1.0.0` (the leading `v` is not part of semver), `1.0.0.0` (too many components), or pre-release identifiers with disallowed characters.

Fix the version in the manifest to a strict semver string. If you are upgrading from a tool that allowed loose versions, normalize all entries — even one bad string in a transitive dep will break resolution.

Lockfile entries should never need manual editing; if you see this for a lockfile entry, regenerate it with `hhs lock` rather than fixing by hand.",
        hints: &["Use exactly three components: MAJOR.MINOR.PATCH", "Do not prefix with `v` — that is a tag convention, not semver", "Pre-release: `1.0.0-rc.1`; build metadata: `1.0.0+build.7`", "Regenerate the lockfile with `hhs lock` if you edited it"],
        example_bad: Some("version = \"v1.0\""),
        example_good: Some("version = \"1.0.0\""),
        see_also: &["PackageVersionNotFound", "PackageDependencyResolution", "PackageToml"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_IO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(173),
        long_code: "HHS_E_PACKAGE_IO",
        short_code: "E0173",
        title: "I/O error in package operation",
        short_description: "An underlying filesystem operation failed while the package manager was reading or writing files.",
        long_description: "The package manager touches a lot of disk: extracting archives, writing the lockfile, populating the package cache, atomically moving directories into place. Any of these can fail with the usual filesystem errors — permission denied, no space left, read-only filesystem, file handle exhaustion, or a parent directory removed mid-operation.

The wrapped IO error has the OS-level reason; read it. If it points at the package cache, check `HHS_HOME` or its equivalent has the right permissions. If it points at the project, your working tree may be on a read-only mount or have stale lockfiles from a crashed previous run.

Delete leftover `.tmp` and `.lock` files inside the cache and retry. If a single package consistently fails to extract, its archive may be corrupt — clear the cache for that package and re-download.",
        hints: &["Read the wrapped IO error for the exact reason", "Check permissions on the package cache directory", "Clean stale `.tmp`/`.lock` files from a crashed previous run", "Free disk space and retry"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageNetwork", "ModelManagerIo", "ModuleLoaderReadError"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_NETWORK: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(174),
        long_code: "HHS_E_PACKAGE_NETWORK",
        short_code: "E0174",
        title: "Package manager network error",
        short_description: "A network operation against the package registry or a download mirror failed.",
        long_description: "The package manager could not complete a network request — to the registry index, to a tarball mirror, or to a Git remote for a git-source dependency. The wrapped error preserves the underlying message: DNS failure, TLS handshake error, connection refused, timeout, or non-2xx HTTP status from the server.

For transient failures, retrying is the simplest fix; the package manager backs off automatically a few times before reporting this. For persistent failures, check whether you are behind a proxy (set `https_proxy`) and whether the registry host is reachable from your network.

For git-source dependencies, make sure you have credentials configured for the host (SSH keys, HTTPS tokens) and that the URL is correct.",
        hints: &["Retry — the package manager already retried briefly", "Set `https_proxy` if you are behind a corporate firewall", "For git dependencies, check SSH keys and remote URLs", "Verify the registry host is reachable: `curl -I <registry>`"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageChecksumMismatch", "HfHttp", "OllamaHttp"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_OTHER: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(175),
        long_code: "HHS_E_PACKAGE_OTHER",
        short_code: "E0175",
        title: "Unspecified package manager error",
        short_description: "An error that does not fit any of the package manager's structured categories was encountered.",
        long_description: "This is the catch-all variant for failures that do not map to a more specific package error. The wrapped message is the only signal you have about the actual cause; read it carefully.

If you see `PackageOther` in production, that is usually a sign the underlying error should have been mapped to a more specific code — please file an issue with the message text so the package manager can be improved. In the meantime, the workaround is whatever the wrapped message suggests.

Do not retry blindly: an opaque error is more often a configuration or environment problem than a transient one.",
        hints: &["Read the wrapped message — that is the only specific signal", "File an issue if the message would benefit from a dedicated code", "Avoid blind retries — opaque errors are rarely transient", "Check recent changes to your manifest or environment"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageBuildFailed", "PackageIo", "PackageNetwork"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_PACKAGE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(176),
        long_code: "HHS_E_PACKAGE_PACKAGE_NOT_FOUND",
        short_code: "E0176",
        title: "Package not found in registry",
        short_description: "The package manager could not find a package with this name in any configured registry.",
        long_description: "The resolver looked up a dependency by name and no configured registry had it. Either the name is misspelled, the registry index is stale, or you are using a registry that does not host this package.

Double-check the spelling and casing in your manifest. Run `hhs update --index` (or your install's equivalent) to refresh the registry index. If the package lives on an alternative registry, declare it in your manifest's registries section so the resolver knows where to look.

For a brand-new package the publisher has not pushed yet, you have to wait for the index to update. For private packages, check that you have the right registry token configured.",
        hints: &["Check spelling and casing in the manifest", "Refresh the registry index with `hhs update --index`", "Declare alternative registries explicitly in the manifest", "Verify your registry token for private packages"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageVersionNotFound", "PackageDependencyResolution", "PackageInvalidPackageName"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_SECURITY_VULNERABILITY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(177),
        long_code: "HHS_E_PACKAGE_SECURITY_VULNERABILITY",
        short_code: "E0177",
        title: "Package has known security vulnerability",
        short_description: "The resolver matched a package version that is listed in the security advisory database, and the policy refuses to install it.",
        long_description: "HudHudScript's package manager consults a security advisory database during resolution and refuses to install versions with known vulnerabilities by default. The error message names the package, the affected version range, and the advisory id so you can read the details.

Upgrade the dependency to a patched version (the advisory usually lists the first fixed release). If no fix is available yet, you can explicitly allow the vulnerability — but only after a deliberate review — by adding it to the project's audit exception list. Never allow unknown advisories blindly.

If the affected package is a transitive dependency, the upgrade may need to happen one or two levels up: bump the direct dependency to a release that pulls in a patched version of the transitive one.",
        hints: &["Upgrade to the first patched version listed in the advisory", "For transitive deps, upgrade the direct dependency that pulls them in", "Add an audit exception only after a documented review", "Never silence advisories you have not read"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageChecksumMismatch", "PackageDependencyResolution", "PackageVersionNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_SERIALIZATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(178),
        long_code: "HHS_E_PACKAGE_SERIALIZATION",
        short_code: "E0178",
        title: "Package data serialization error",
        short_description: "The package manager failed to serialize or deserialize a structured payload (registry response, lockfile, manifest cache).",
        long_description: "The package manager exchanges structured data with several subsystems — JSON for the registry index, TOML for manifests, and a binary form for the on-disk cache. This error means one of those serialization layers failed: a registry returned malformed JSON, a cached entry was written by an older version, or a manifest had an unexpected shape.

The wrapped serde error names the offending field. For cache errors, deleting the cache and re-running is the simplest fix. For registry errors, the registry may be down or returning an error envelope; reproduce with `curl` to confirm.

This is a different code than `PackageToml` (which is for TOML-specific parse errors); `PackageSerialization` covers the broader serialization layer.",
        hints: &["Read the wrapped serde error to find the offending field", "Delete the package cache if the failure is on a cached entry", "For registry errors, reproduce the request with `curl`", "Upgrade the package manager if the schema has changed"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageToml", "PackageTomlSerialize", "HfDeserialize"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_TOML: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(179),
        long_code: "HHS_E_PACKAGE_TOML",
        short_code: "E0179",
        title: "Failed to parse TOML",
        short_description: "A TOML file (manifest or lockfile) could not be parsed because of a syntax error.",
        long_description: "The TOML parser rejected a manifest or lockfile. The wrapped error contains the line, column, and human description of the problem — read it first; this outer error just identifies the file that failed.

Common mistakes: unquoted values that contain reserved characters, missing closing brackets in array-of-tables, mixing tab and space indentation in deeply nested tables, and the classic 'used `=` where `:` is expected' (or vice versa). TOML is stricter than JSON about table headers and key uniqueness, so accidentally declaring the same `[dependencies]` section twice will trip this.

For lockfiles, do not edit them by hand — regenerate with `hhs lock`. For manifests, run them through a TOML linter if the parser message is unclear.",
        hints: &["Read the wrapped parse error for line and column", "Do not edit lockfiles by hand — regenerate with `hhs lock`", "Watch for duplicate `[dependencies]` headers", "Quote values with special characters"],
        example_bad: Some("[dependencies]
foo = 1.0  # bare float, parser sees an unquoted version"),
        example_good: Some("[dependencies]
foo = \"1.0.0\""),
        see_also: &["PackageTomlSerialize", "PackageSerialization", "PackageInvalidVersion"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_TOML_SERIALIZE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(180),
        long_code: "HHS_E_PACKAGE_TOML_SERIALIZE",
        short_code: "E0180",
        title: "Failed to write TOML",
        short_description: "The package manager could not serialize a structure into TOML when writing a manifest or lockfile.",
        long_description: "The package manager generates TOML files (lockfile, scratch manifests). Serialization can fail if a struct contains values that have no TOML representation — most commonly, a map with non-string keys, or a heterogeneous array of tables ordered in a way TOML cannot express.

This error almost always indicates a bug in the package manager rather than user input, because the structures it serializes are produced by its own code. If you see it, please file an issue with the operation that triggered it.

In the meantime, the wrapped error names the offending field; if you can edit the corresponding manifest by hand to avoid the problematic shape, that is a viable workaround.",
        hints: &["Read the wrapped error — it names the field that failed", "This usually indicates a package manager bug; consider filing an issue", "Workaround: hand-edit the manifest to avoid the problematic shape", "Avoid unusual key types or nested array-of-table mixes"],
        example_bad: None,
        example_good: None,
        see_also: &["PackageToml", "PackageSerialization", "PackageIo"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const PACKAGE_VERSION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(181),
        long_code: "HHS_E_PACKAGE_VERSION_NOT_FOUND",
        short_code: "E0181",
        title: "No matching version in registry",
        short_description: "The package exists in the registry, but no published version satisfies the requested version constraint.",
        long_description: "The resolver found the package by name but could not find a published version inside the constraint range you specified. Common causes: a typo'd version range, a constraint that has not been released yet, or a release that was yanked from the registry between your last successful install and now.

Loosen the constraint to a range the registry actually contains, or update the registry index in case the version you want has been published since your local index was refreshed (`hhs update --index`).

If the version you depend on was yanked, that is a signal not to use it — pick the next-newest unyanked version instead.",
        hints: &["Refresh the index: `hhs update --index`", "List published versions: `hhs search <pkg>`", "Loosen overly tight constraints (`=1.2.3` -> `^1.2`)", "Yanked versions are intentionally excluded — pick another"],
        example_bad: Some("[dependencies]
foo = \"=99.0.0\""),
        example_good: Some("[dependencies]
foo = \"^1.2\""),
        see_also: &["PackagePackageNotFound", "PackageDependencyResolution", "PackageInvalidVersion"],
        since_version: "0.4.0",
        category: ErrorCategory::Package,
    };

pub const RESOLVER_INVALID_PATH: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(215),
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
        category: ErrorCategory::Package,
    };

pub const RESOLVER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(216),
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
        category: ErrorCategory::Package,
    };
