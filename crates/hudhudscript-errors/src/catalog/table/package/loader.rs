use super::*;
use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const MODULE_LOADER_ALREADY_LOADED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(131),
        long_code: "HHS_E_MODULE_LOADER_ALREADY_LOADED",
        short_code: "E0131",
        title: "Module already loaded",
        short_description: "An explicit load was requested for a module that is already in the loader's cache, and the API refuses to reload it.",
        long_description: "The module loader caches every module by canonical path and considers re-loading an error rather than a no-op for APIs that demand a fresh load (such as test isolation harnesses or hot-reload entry points). This avoids surprising aliasing where two parts of the program think they own different copies of the same module state.

For normal `import` statements you should never see this error — the loader returns the cached instance silently. If you are using a programmatic loader API, switch to the `get_or_load` variant, or unload the module first with the corresponding `unload` call.

During development with hot-reload, this error sometimes indicates that an earlier reload failed half-way and left a stale entry; use `hhs reload --force` to clear the cache.",
        hints: &["Use `get_or_load` instead of `load` for idempotent loading", "Call `unload` before re-loading if you really need a fresh copy", "For hot-reload issues, try `hhs reload --force`", "Avoid mixing programmatic loads with `import` for the same module"],
        example_bad: None,
        example_good: None,
        see_also: &["ModuleLoaderModuleNotFound", "ModelManagerAlreadyExists", "GraphModuleNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODULE_LOADER_MODULE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(132),
        long_code: "HHS_E_MODULE_LOADER_MODULE_NOT_FOUND",
        short_code: "E0132",
        title: "Module file not found by loader",
        short_description: "The loader could not find a file matching the import path on disk after applying its search rules.",
        long_description: "The module loader resolves an import like `import { x } from \"./util\"` to a concrete file by trying the path with known extensions (.hhs) and walking parent directories for package roots. This error means none of those candidates exist on disk.

Check the spelling of the import path, that the file exists where you expect, and that you are not relying on case-insensitive matching on a case-sensitive filesystem (Linux/macOS-with-APFS-case-sensitive). For relative imports, remember they resolve relative to the importing file, not the current working directory.

If the module belongs to a dependency, make sure the dependency is declared in your manifest and has been installed (`hhs install`).",
        hints: &["Verify the file exists at the resolved path", "Check casing — Linux filesystems are case-sensitive", "Relative imports are relative to the importing file, not cwd", "Run `hhs install` to fetch missing dependencies"],
        example_bad: Some("import { parse } from \"./Util\" // file is util.hhs on case-sensitive fs"),
        example_good: Some("import { parse } from \"./util\""),
        see_also: &["ModuleLoaderReadError", "GraphModuleNotFound", "ResolverNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODULE_LOADER_PARSE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(133),
        long_code: "HHS_E_MODULE_LOADER_PARSE_ERROR",
        short_code: "E0133",
        title: "Failed to parse module source",
        short_description: "The loader found the module file but its contents could not be parsed as valid HudHudScript.",
        long_description: "The file was located and read successfully, but the parser rejected its contents. The wrapped diagnostic carries the actual syntax error and source location — read it first; this outer error just tells you which module the parse failed in.

Common causes are: editing the file while the loader was running, an incomplete Git merge that left conflict markers in the source, a non-UTF-8 file (the parser requires UTF-8), or a missing dependency that the parser would have needed for macro expansion.

This error is reported during the parse-time graph build, before any code from the module runs. Fix the syntax error and re-run.",
        hints: &["Read the wrapped diagnostic — it points at the exact line/column", "Check for unresolved Git merge conflict markers", "Ensure the file is saved as UTF-8", "Run `hhs check` to see all parse errors at once"],
        example_bad: None,
        example_good: None,
        see_also: &["ModuleLoaderReadError", "GraphCircularDependency", "ModuleLoaderModuleNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODULE_LOADER_READ_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(134),
        long_code: "HHS_E_MODULE_LOADER_READ_ERROR",
        short_code: "E0134",
        title: "Failed to read module file",
        short_description: "The loader could not read the module file from disk due to an underlying filesystem error.",
        long_description: "Unlike `ModuleNotFound`, this error means the file exists in the directory listing but the loader could not read its bytes. Typical causes are permission denied, the file being unreadable for the user, an I/O error mid-read on a failing disk, or the file being a directory or special device that was matched by the resolver.

The wrapped IO error has the operating system's exact reason; read it. Permission denied is fixed with `chmod`; 'is a directory' usually points at a resolver bug or a missing extension on the import path.

If the file lives on a network filesystem, transient timeouts are common — retrying once is often enough to confirm whether the cause is persistent.",
        hints: &["Read the wrapped IO error for the specific reason", "Check file permissions: `ls -l <file>`", "If on NFS/SMB, retry to rule out transient timeouts", "Make sure the import path resolved to a file, not a directory"],
        example_bad: None,
        example_good: None,
        see_also: &["ModuleLoaderModuleNotFound", "ModuleLoaderParseError", "PackageIo"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const OLLAMA_DESERIALIZE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(155),
        long_code: "HHS_E_OLLAMA_DESERIALIZE",
        short_code: "E0155",
        title: "Failed to deserialize Ollama response",
        short_description: "The Ollama API returned a body that does not match the expected JSON schema for this endpoint.",
        long_description: "The model manager called the local Ollama daemon and could not deserialize its response. Ollama's HTTP API is generally stable but occasionally adds or renames fields between versions, and you may be running a build of HudHudScript that expects a different shape than the daemon you have installed.

Reproduce the request with `curl http://localhost:11434/api/...` and inspect the body. If it has new or missing fields compared to what HudHudScript expects, upgrade either side until they match.

A second common cause is that the response is a streaming NDJSON body but the caller treated it as a single JSON object (or vice versa). The model manager picks the right mode per endpoint, so if you see this on a streaming endpoint, it usually points at a daemon-side error frame in the middle of the stream.",
        hints: &["Reproduce the request with `curl` and inspect the JSON body", "Check `ollama --version` against the version HudHudScript expects", "Streaming endpoints return NDJSON; non-streaming return a single object", "Upgrade Ollama or HudHudScript to align the schemas"],
        example_bad: None,
        example_good: None,
        see_also: &["OllamaHttp", "HfDeserialize", "PackageSerialization"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const OLLAMA_HTTP: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(156),
        long_code: "HHS_E_OLLAMA_HTTP",
        short_code: "E0156",
        title: "HTTP request to Ollama failed",
        short_description: "The HTTP client could not reach the Ollama daemon, or the daemon returned a non-2xx status.",
        long_description: "The model manager could not complete an HTTP call to Ollama (default `http://localhost:11434`). The wrapped error tells you whether the daemon was unreachable (connection refused — daemon not running), returned a 404 (model not pulled), 500 (daemon-side failure), or hit a timeout (long generation without keep-alive).

For connection refused, start Ollama with `ollama serve` or check that the systemd unit is up. For 404, run `ollama pull <model>` to install the model first. For timeouts, increase the request timeout in the model manager config or generate in smaller chunks.

If Ollama is bound to a non-default address, set the `OLLAMA_HOST` environment variable so the model manager uses the right URL.",
        hints: &["Start the daemon: `ollama serve`", "Pull missing models: `ollama pull <name>`", "Set OLLAMA_HOST if the daemon binds to a non-default address", "Increase timeouts for long generation requests"],
        example_bad: None,
        example_good: None,
        see_also: &["OllamaDeserialize", "HfHttp", "PackageNetwork"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };
