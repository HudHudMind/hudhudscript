use super::*;
use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const GGUF_INVALID_MAGIC: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(82),
        long_code: "HHS_E_GGUF_INVALID_MAGIC",
        short_code: "E0082",
        title: "GGUF file has invalid magic bytes",
        short_description: "The first four bytes of the file are not the GGUF magic header (0x47 0x47 0x55 0x46), so this is not a valid GGUF model.",
        long_description: "Every GGUF file begins with the four ASCII bytes 'GGUF' (0x47475546). The loader rejected this file because that header was missing or corrupted. The most common causes are: a truncated download, an HTML error page saved with a .gguf extension, an unrelated binary (PyTorch .bin, safetensors, ONNX) renamed to .gguf, or accidental text-mode conversion that mangled the first bytes.

To recover, re-download the model from a trusted source and verify its size and SHA-256 against the publisher's record. If you used the HuggingFace loader, prefer the canonical revision pin instead of 'main', because mirror servers can return HTML error pages when rate-limited.

GGUF is the on-disk format used by llama.cpp and ggml-based runtimes. HudHudScript's model loader memory-maps the file and inspects the header before doing any further parsing, which is why this is the very first error you will see for a malformed file.",
        hints: &["Run `file path/to/model.gguf` — it should report 'GGUF model'", "Re-download the model; the previous transfer was likely truncated", "Verify the SHA-256 checksum against the publisher's listing", "Make sure you did not rename a .bin/.safetensors file to .gguf"],
        example_bad: Some("load_model(\"./tinyllama.bin\") // PyTorch checkpoint renamed to .gguf"),
        example_good: Some("load_model(\"./tinyllama-q4_k_m.gguf\") // real GGUF file from llama.cpp"),
        see_also: &["GgufTooShort", "GgufUnsupportedVersion", "GgufUnexpectedEof"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const GGUF_INVALID_UTF8: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(83),
        long_code: "HHS_E_GGUF_INVALID_UTF8",
        short_code: "E0083",
        title: "GGUF metadata contains invalid UTF-8",
        short_description: "A metadata key or string value inside the GGUF file is not valid UTF-8 and cannot be decoded.",
        long_description: "GGUF metadata strings are length-prefixed UTF-8 byte sequences. The loader hit a byte sequence that does not decode as UTF-8 — typically a truncated multi-byte character at a buffer boundary, a corrupted middle-of-file region, or a file produced by a non-conformant converter that wrote raw Latin-1 bytes.

The fix is almost always to re-download or regenerate the file. If you converted the model yourself with `convert.py` from llama.cpp, upgrade to the current version — older converters had bugs that wrote bad bytes for non-ASCII tokenizer entries (Chinese, Arabic, emoji).

This error does not indicate a bug in HudHudScript: the loader is doing strict UTF-8 validation deliberately, because tokenizer vocabularies are written into metadata and an invalid string here would silently corrupt every prompt later.",
        hints: &["Re-download the model; the file is corrupted", "If you converted it yourself, use the latest llama.cpp convert.py", "Check disk health — silent bit-rot can produce this error", "Compare the file's SHA-256 against the publisher's record"],
        example_bad: None,
        example_good: None,
        see_also: &["GgufInvalidMagic", "GgufUnexpectedEof", "HfDeserialize"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const GGUF_TOO_SHORT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(84),
        long_code: "HHS_E_GGUF_TOO_SHORT",
        short_code: "E0084",
        title: "GGUF file shorter than header size",
        short_description: "The file is smaller than the minimum GGUF header (magic + version + tensor count + kv count), so it cannot be a valid model.",
        long_description: "A valid GGUF file is at least the size of its fixed header — magic bytes plus three little-endian u64 fields. This file is shorter than that minimum, so the loader bailed out before doing any real work.

This usually means the download was aborted within the first few hundred bytes (network reset, disk full mid-write, or you tried to load an empty placeholder). It can also happen if a build script created a stub file before the real download completed.

Delete the file and re-fetch it. If you are using the model manager's resume-download feature, clear the partial state with `hhs model clean <name>` before retrying so the resume offset is reset.",
        hints: &["Delete the partial file and re-download from scratch", "Check available disk space before downloading multi-GB models", "Use `hhs model clean <name>` to reset partial-download state", "Verify the file size matches the publisher's listing"],
        example_bad: None,
        example_good: None,
        see_also: &["GgufInvalidMagic", "GgufUnexpectedEof", "ModelManagerInsufficientDiskSpace"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const GGUF_UNEXPECTED_EOF: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(85),
        long_code: "HHS_E_GGUF_UNEXPECTED_EOF",
        short_code: "E0085",
        title: "GGUF parser hit unexpected end of file",
        short_description: "The header declared more data (tensors, metadata kvs) than the file actually contains, so parsing ran past the end.",
        long_description: "GGUF headers list how many tensors and metadata key-value pairs follow. While reading them, the parser ran out of bytes — meaning the file is truncated. The header itself was valid, which is why this manifests as a different error than `GgufTooShort`.

The usual cause is an interrupted download that completed enough bytes to look plausible but stopped before the tensor block. Some HTTP intermediaries also cap large responses; if you fetched the model through a corporate proxy, check its body-size limit.

Re-download with a tool that verifies content length, or use the HudHudScript model manager which tracks expected size in its catalog and refuses to mark a download complete unless every byte arrived.",
        hints: &["Re-download the file; it is truncated", "Compare local file size with `Content-Length` from the source", "Check proxy/firewall body-size limits for large model downloads", "Prefer `hhs model pull` which validates length on completion"],
        example_bad: None,
        example_good: None,
        see_also: &["GgufTooShort", "GgufInvalidMagic", "HfHttp"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const GGUF_UNSUPPORTED_VERSION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(86),
        long_code: "HHS_E_GGUF_UNSUPPORTED_VERSION",
        short_code: "E0086",
        title: "GGUF file uses an unsupported version",
        short_description: "The file's GGUF version field is newer or older than the versions this build of HudHudScript can read.",
        long_description: "GGUF is a versioned format (v1, v2, v3, ...). HudHudScript supports a fixed range and refuses files outside it rather than risking silent misinterpretation. If you got a v3 file but the runtime only knows v2, the tensor offset table layout differs and reading would corrupt tensors.

Upgrade HudHudScript to a release that supports the newer version, or re-quantize the model with an older converter that targets the version you do support. The publisher's model card usually lists the GGUF version produced.

If you cannot upgrade, llama.cpp's `gguf-py` ships a converter that can downgrade some metadata-only differences, but tensor layout changes between major versions cannot be downgraded automatically.",
        hints: &["Upgrade HudHudScript to a build that supports the file's GGUF version", "Re-quantize the model with a matching llama.cpp converter version", "Check the model card for the GGUF version it was produced with", "Run `hhs --version` to see which GGUF range your build supports"],
        example_bad: None,
        example_good: None,
        see_also: &["GgufInvalidMagic", "GgufUnexpectedEof", "ModelManagerNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const GRAPH_CIRCULAR_DEPENDENCY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(102),
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
        category: ErrorCategory::Package,
    };

pub const GRAPH_MODULE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(103),
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
        category: ErrorCategory::Package,
    };

pub const HF_DESERIALIZE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(104),
        long_code: "HHS_E_HF_DESERIALIZE",
        short_code: "E0104",
        title: "Failed to deserialize HuggingFace response",
        short_description: "The HuggingFace API returned a response body that does not match the expected JSON schema for this endpoint.",
        long_description: "The model manager called the HuggingFace Hub API and got back JSON it could not deserialize into the expected struct. This usually happens when: the Hub changed its response format on a particular endpoint, you hit a rate-limit page that returned HTML instead of JSON, or you queried a private repo without an access token and got back an error envelope instead of the expected payload.

First verify you can fetch the same URL with `curl` and that the body looks like the JSON the loader expects. If the body is HTML or a `{\"error\": ...}` envelope, the underlying problem is authentication or rate-limiting, not deserialization.

If the HuggingFace API has genuinely shifted (which they do occasionally), upgrade HudHudScript or pin to a known-good revision while waiting for a fix.",
        hints: &["Reproduce the request with `curl` and inspect the body", "Set HF_TOKEN if you are accessing private or gated repos", "Check HuggingFace status if many requests fail at once", "Upgrade HudHudScript if the API schema has changed"],
        example_bad: None,
        example_good: None,
        see_also: &["HfHttp", "OllamaDeserialize", "PackageSerialization"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const HF_HTTP: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(105),
        long_code: "HHS_E_HF_HTTP",
        short_code: "E0105",
        title: "HTTP request to HuggingFace failed",
        short_description: "The HTTP client returned an error talking to the HuggingFace Hub: DNS, TLS, connection refused, timeout, or non-2xx status.",
        long_description: "The model manager could not complete an HTTP request to huggingface.co. The wrapped error is preserved verbatim so you can see whether it was DNS, TLS, a 401 (auth), a 403 (gated repo without acceptance), a 404 (typo'd repo id), a 429 (rate-limited), or a 5xx outage on the Hub side.

For 401/403, set `HF_TOKEN` to a valid access token and, for gated models, accept the license on the model page first. For 404, double-check the repo id (`org/name`, case-sensitive). For 429, back off and retry with exponential delay. For network-level errors, check your proxy settings and that `huggingface.co` is reachable.

The model manager retries transient failures internally a few times before surfacing this error, so by the time you see it the failure is persistent within that window.",
        hints: &["Set HF_TOKEN for private or gated repositories", "Verify the repo id is exactly `org/name` (case-sensitive)", "Check `https_proxy` if you are behind a corporate firewall", "On 429, back off and retry; the Hub rate-limits unauthenticated calls"],
        example_bad: None,
        example_good: None,
        see_also: &["HfDeserialize", "OllamaHttp", "PackageNetwork"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODEL_MANAGER_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(127),
        long_code: "HHS_E_MODEL_MANAGER_ALREADY_EXISTS",
        short_code: "E0127",
        title: "Model already registered in catalog",
        short_description: "A model with this name already exists in the local catalog and the operation refuses to overwrite it.",
        long_description: "The model manager keeps a catalog of installed models keyed by name. You tried to register a new entry with a name that is already taken, and the API in question (e.g. `register`, `import`) is non-destructive by design — it does not silently overwrite.

If you intended to replace the existing entry, remove it first with `hhs model remove <name>` or use the explicit force/replace flag of the operation you were calling. If you intended to register a different version, give it a distinct name (`llama3-8b-q4` vs `llama3-8b-q8`) so both can coexist.

This error is informational, not a corruption — your existing model is untouched and still usable.",
        hints: &["Remove the existing entry with `hhs model remove <name>` first", "Use a distinct name for variants (e.g. include the quantization)", "Use the explicit `--replace` flag if your operation supports it", "List installed models with `hhs model list`"],
        example_bad: None,
        example_good: None,
        see_also: &["ModelManagerNotFound", "ModelManagerIo", "ModuleLoaderAlreadyLoaded"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODEL_MANAGER_INSUFFICIENT_DISK_SPACE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(128),
        long_code: "HHS_E_MODEL_MANAGER_INSUFFICIENT_DISK_SPACE",
        short_code: "E0128",
        title: "Not enough disk space for model",
        short_description: "The download cannot proceed because the target volume has less free space than the model's expected size plus a safety margin.",
        long_description: "Before starting a download, the model manager queries free space on the destination volume and compares it against the expected file size (with a small overhead for temporary unpacking). This check failed, so no bytes were written.

Free space on the destination, point the cache to a larger volume by setting `HHS_MODEL_CACHE`, or pick a smaller quantization (q4_0, q4_k_m) instead of q8_0/f16. Multi-billion parameter models can easily consume tens of gigabytes per checkpoint, so picking the right quantization is often the right answer.

The error message includes both the required and available byte counts so you can see exactly how much more space you need.",
        hints: &["Free space on the destination volume", "Set HHS_MODEL_CACHE to a path on a larger disk", "Pick a smaller quantization (q4_k_m instead of f16)", "Run `hhs model gc` to evict unused cached models"],
        example_bad: None,
        example_good: None,
        see_also: &["ModelManagerIo", "GgufTooShort", "PackageIo"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODEL_MANAGER_IO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(129),
        long_code: "HHS_E_MODEL_MANAGER_IO",
        short_code: "E0129",
        title: "I/O error in model manager",
        short_description: "An underlying filesystem operation (read, write, rename, mkdir) failed while managing a model on disk.",
        long_description: "The model manager performs many filesystem operations: creating cache directories, atomically renaming downloaded files into place, computing checksums, and reading metadata. Any of these can fail for the usual reasons — permission denied, read-only filesystem, file-handle exhaustion, transient network filesystem timeout, or a parent directory that disappeared between checks.

The wrapped `std::io::Error` carries the original message; read it carefully — `Permission denied`, `No such file or directory`, and `Read-only file system` each point at very different fixes.

If the cache lives on a network mount (NFS, SMB), consider moving it to a local disk: many model files are large and concurrent writers behave poorly on networked filesystems.",
        hints: &["Read the wrapped IO error message — it names the exact failure", "Check permissions on `$HHS_MODEL_CACHE`", "Avoid placing the model cache on NFS/SMB if possible", "Run `hhs model doctor` to inspect cache integrity"],
        example_bad: None,
        example_good: None,
        see_also: &["ModelManagerInsufficientDiskSpace", "PackageIo", "ModuleLoaderReadError"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };

pub const MODEL_MANAGER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(130),
        long_code: "HHS_E_MODEL_MANAGER_NOT_FOUND",
        short_code: "E0130",
        title: "Model not found in catalog",
        short_description: "No model with this name is registered in the local catalog or available from any configured remote source.",
        long_description: "You asked the model manager to load or operate on a model that is neither in the local catalog nor known to any configured remote (HuggingFace, Ollama, custom registry). Either the name is misspelled, the model has not been downloaded yet, or the remote source is not configured.

List what is installed with `hhs model list`, and use `hhs model search <pattern>` to query remotes. To install a model from HuggingFace, use `hhs model pull hf:org/repo` (substitute the actual loader prefix your installation uses).

If you expect the model to be installed, check that you are running with the same `HHS_MODEL_CACHE` as when you installed it — different processes pointing at different caches will report different catalogs.",
        hints: &["Run `hhs model list` to see installed models", "Pull the model first: `hhs model pull <name>`", "Verify HHS_MODEL_CACHE matches the install environment", "Check spelling — model names are case-sensitive"],
        example_bad: None,
        example_good: None,
        see_also: &["ModelManagerAlreadyExists", "GraphModuleNotFound", "ResolverNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Package,
    };
