use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const GGUF_INVALID_MAGIC: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(82),
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
        category: ExceptionCategory::Package,
    };

pub const GGUF_INVALID_UTF8: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(83),
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
        category: ExceptionCategory::Package,
    };

pub const GGUF_TOO_SHORT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(84),
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
        category: ExceptionCategory::Package,
    };

pub const GGUF_UNEXPECTED_EOF: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(85),
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
        category: ExceptionCategory::Package,
    };

pub const GGUF_UNSUPPORTED_VERSION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(86),
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
        category: ExceptionCategory::Package,
    };
