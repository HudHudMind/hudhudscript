use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const MODEL_MANAGER_ALREADY_EXISTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(127),
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
        category: ExceptionCategory::Package,
    };

pub const MODEL_MANAGER_INSUFFICIENT_DISK_SPACE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(128),
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
        category: ExceptionCategory::Package,
    };

pub const MODEL_MANAGER_IO: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(129),
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
        category: ExceptionCategory::Package,
    };

pub const MODEL_MANAGER_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(130),
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
        category: ExceptionCategory::Package,
    };
