use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 13] = [
    ExceptionEntry {
        code: ExceptionCode(11),
        long_code: "HHS_E_BRIDGE_CONNECTION_LOST",
        short_code: "E0011",
        title: "UI Bridge Lost Its Connection",
        short_description: "The transport between HudHudScript and the host UI framework was severed after a successful initial handshake.",
        long_description: "The UI bridge in `hudhudscript-ui-core` keeps a long-lived channel to the embedded framework (Tauri, Dioxus, web view, etc.). Once initialized, both sides exchange render commands and event callbacks over this channel. When the channel closes unexpectedly — usually because the host window was destroyed, the IPC pipe broke, or the framework process crashed — the bridge raises this error and refuses further calls.

Unlike `BridgeInitFailed`, the connection was healthy at some point. Anything you queued after the disconnect is dropped, and pending awaits resolve to this error.

Decide whether to recreate the bridge from scratch or to surface the failure to the user. Auto-reconnect is not performed because it can mask deeper crashes in the host framework.",
        hints: &["Check whether the host window or webview was closed by the user", "Inspect host-process logs for crashes around the disconnect time", "Wrap long-lived UI sessions in a supervisor that recreates the bridge", "Cancel pending UI awaits when this error fires to avoid leaks"],
        example_bad: None,
        example_good: None,
        see_also: &["BridgeInitFailed", "BridgeFrameworkError", "BridgeRenderFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(12),
        long_code: "HHS_E_BRIDGE_FRAMEWORK_ERROR",
        short_code: "E0012",
        title: "Underlying UI Framework Reported Error",
        short_description: "The host framework returned an error while handling a bridge request relayed from HudHudScript.",
        long_description: "When the bridge forwards a call into Tauri, Dioxus, the web view, or any other supported backend, that backend may reject the call for its own reasons — invalid widget id, unsupported property, layout constraint violation, and so on. The bridge wraps the underlying message verbatim so you do not lose the framework-specific context.

This variant is distinct from `BridgeRenderFailed`, which is reserved for errors during the actual draw step. `FrameworkError` covers everything else the backend can complain about.

The wrapped string is the primary signal — it comes straight from the framework. Search the host framework's documentation using that text before assuming the bug is in HudHudScript.",
        hints: &["Read the wrapped message — it is verbatim from the host framework", "Cross-reference framework-specific docs for the error text", "Confirm the widget tree matches what the framework expects", "Reproduce against a minimal scene to isolate the offending call"],
        example_bad: None,
        example_good: None,
        see_also: &["BridgeRenderFailed", "BridgeInitFailed", "BridgeUnsupported"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(13),
        long_code: "HHS_E_BRIDGE_INIT_FAILED",
        short_code: "E0013",
        title: "UI Bridge Initialization Failed",
        short_description: "The bridge could not complete its handshake with the selected UI framework during startup.",
        long_description: "Initialization covers loading the framework, allocating the host window or webview, and exchanging the initial capability descriptors with HudHudScript. Any failure in that sequence — missing native dependency, denied permission, invalid configuration, port conflict — surfaces as `InitFailed`.

The bridge has not yet entered the running state, so no UI commands have been sent. Recovery means correcting the underlying cause and trying again; you cannot patch around it from script code once the error has been emitted.

Check installation prerequisites for the chosen backend, look at the wrapped message for OS-level details, and confirm that the binary was built with the requested framework feature flag enabled.",
        hints: &["Verify native dependencies for the selected backend are installed", "Check that the binary was built with the framework feature enabled", "Inspect the wrapped message for OS-level permission issues", "Try a different backend to isolate platform vs configuration"],
        example_bad: None,
        example_good: None,
        see_also: &["BridgeUnsupported", "BridgeConnectionLost", "BridgeFrameworkError"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(14),
        long_code: "HHS_E_BRIDGE_RENDER_FAILED",
        short_code: "E0014",
        title: "UI Bridge Render Step Failed",
        short_description: "The host framework rejected a render command issued by HudHudScript after a successful bridge initialization.",
        long_description: "Render failures happen during the actual draw or layout pass: an invalid drawable, a node that does not fit its parent, a shader compile error, or a backend that ran out of GPU resources. The bridge converts the framework-specific failure into this single variant while preserving the original message.

Because rendering is downstream of state updates, a render failure usually indicates that an earlier state mutation produced an inconsistent tree. The bridge does not auto-rollback — the next frame will retry with whatever state currently exists.

Capture the failing scene description, look for impossible constraints, and consider gating expensive draw paths behind capability checks.",
        hints: &["Look for impossible layout constraints or zero-sized nodes", "Check GPU memory if shaders or large textures are involved", "Reproduce against a minimal widget tree to isolate the trigger", "Add capability checks before invoking optional draw features"],
        example_bad: None,
        example_good: None,
        see_also: &["BridgeFrameworkError", "BridgeConnectionLost", "BridgeInitFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(15),
        long_code: "HHS_E_BRIDGE_UNSUPPORTED",
        short_code: "E0015",
        title: "Requested UI Framework Not Built In",
        short_description: "The script asked for a UI framework that this HudHudScript binary was not compiled to support.",
        long_description: "HudHudScript supports several UI backends behind Cargo feature flags. A given binary only contains the backends that were enabled at build time, so requesting Tauri from a build that only includes the web bridge — or vice versa — yields this error.

This is a build-time decision, not a runtime one: you cannot plug in additional backends after the fact. The error fires before any handshake or window allocation, so no resources are leaked.

Either rebuild the binary with the correct feature flags or change your script to target a backend that is already present. Calling `hhs --version` typically reveals which feature set is active.",
        hints: &["Rebuild with the appropriate `--features ui-tauri` (or similar)", "Run `hhs --version` to see which backends are compiled in", "Switch the script to a backend that is already supported", "Use a feature-detection helper before requesting an optional backend"],
        example_bad: None,
        example_good: None,
        see_also: &["BridgeInitFailed", "BridgeFrameworkError", "BridgeConnectionLost"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(70),
        long_code: "HHS_E_DEPLOY_ADAPTER_ERROR",
        short_code: "E0070",
        title: "Deploy Adapter Reported Failure",
        short_description: "A target-specific deploy adapter (Vercel, Netlify, S3, custom, etc.) returned an error during the deploy pipeline.",
        long_description: "`hudhudscript-deploy-core` dispatches to a pluggable adapter once the build is complete. Each adapter speaks the protocol of its target platform — uploading bundles, calling provider APIs, registering routes. When that adapter fails for any reason it does not classify itself, this variant carries the wrapped message back to the user.

Because the adapter sits between deploy-core and the external service, the cause might be local (bad credentials, missing CLI tool) or remote (service outage, quota). The wrapped text is your fastest signal.

Read the message, check the adapter's documentation, and confirm any required credentials or environment variables are present. Re-run with verbose logging if the message is too terse.",
        hints: &["Read the wrapped message — it comes from the target adapter", "Verify required credentials and environment variables are present", "Re-run with verbose logging for more context", "Check the target service's status page if the cause looks remote"],
        example_bad: None,
        example_good: None,
        see_also: &["DeployBuildFailed", "DeployConfigError", "DeployDeployFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(71),
        long_code: "HHS_E_DEPLOY_BUILD_FAILED",
        short_code: "E0071",
        title: "Deploy Build Step Failed",
        short_description: "The build step preceding the actual deploy could not produce a deployable artifact.",
        long_description: "Before any adapter is invoked, deploy-core builds the project into the format the target expects. A build failure here can come from the HudHudScript compiler, the wrapped front-end build (e.g. Vite, Trunk), or a missing tool in the path. The deploy is aborted before any artifact is uploaded.

The wrapped message usually identifies which build sub-step failed. If the build works locally but fails inside deploy, suspect environment differences — Node version, Rust toolchain, missing native libraries — rather than the project source.

Reproduce the failing build outside the deploy command first. Once the standalone build is green, retry the deploy.",
        hints: &["Reproduce the build standalone before re-running the deploy", "Compare local and CI tool versions when behavior differs", "Inspect the wrapped message for the failing sub-step", "Check for missing native dependencies in the deploy environment"],
        example_bad: None,
        example_good: None,
        see_also: &["DeployAdapterError", "DeployConfigError", "DeployDeployFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(72),
        long_code: "HHS_E_DEPLOY_CONFIG_ERROR",
        short_code: "E0072",
        title: "Deploy Configuration Invalid Or Missing",
        short_description: "The deploy command could not load or validate its configuration, including the project directory and target settings.",
        long_description: "Deploy-core requires a configuration that names the project directory, the build target, and any adapter-specific settings. This error fires when that configuration cannot be located, fails schema validation, or contradicts itself (for example, asking for a static target while declaring server-side rendering).

The most common variant is a missing or non-existent `--project-dir`, which is required as of recent releases. Other causes include invalid target names, malformed config files, and environment variables that override config in unexpected ways.

Correct the configuration and re-run. Use `--help` to see which fields are required and which have defaults.",
        hints: &["Verify `--project-dir` exists and contains a valid project", "Check the build target name against the supported list", "Watch for env vars overriding your config silently", "Validate config files against the documented schema"],
        example_bad: Some("hhs deploy --project-dir ./does-not-exist"),
        example_good: Some("hhs deploy --project-dir ./my-app --target static"),
        see_also: &["DeployBuildFailed", "DeployAdapterError", "DeployDeployFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(73),
        long_code: "HHS_E_DEPLOY_DEPLOY_FAILED",
        short_code: "E0073",
        title: "Deploy Step Failed After Successful Build",
        short_description: "The artifact was built successfully but the actual deploy step (upload, activation, switchover) did not complete.",
        long_description: "This variant covers the final phase of the pipeline: pushing the built artifact to its destination and making it live. Failures here are usually network, authentication, or quota related, but can also come from server-side validation rejecting the artifact (wrong format, missing manifest field, etc.).

Unlike `DeployAdapterError`, which is the generic adapter wrapper, `DeployFailed` specifically marks the deploy phase. Earlier phases — config, build — succeeded.

Check connectivity, credentials, and quotas. Re-run with verbose logging if the message lacks detail. Consider whether the previous version is still serving traffic or whether a partial deploy left the target in a degraded state.",
        hints: &["Verify network connectivity and credentials to the target", "Check whether you hit a quota or rate limit on the target service", "Inspect server-side logs if the artifact was rejected after upload", "Confirm whether a partial deploy needs manual cleanup"],
        example_bad: None,
        example_good: None,
        see_also: &["DeployRollbackFailed", "DeployAdapterError", "DeployBuildFailed"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(74),
        long_code: "HHS_E_DEPLOY_ROLLBACK_FAILED",
        short_code: "E0074",
        title: "Deploy Rollback Could Not Complete",
        short_description: "After a failed deploy, the automatic rollback to the previous good version did not succeed and the target may be degraded.",
        long_description: "When a deploy fails partway through, deploy-core attempts to restore the previous good version so the target keeps serving traffic. This variant fires when that rollback itself fails — usually because the previous artifact is missing, the target API rejects the rollback call, or credentials lapsed between the deploy and the rollback attempt.

This is one of the most serious deploy errors because the live target is now in an undefined state. The pipeline does not retry on its own; manual intervention is expected.

Check the target manually, restore the previous version by hand if needed, and only then re-run the deploy. Keep an out-of-band record of the last known good artifact so manual rollback is always possible.",
        hints: &["Treat this as urgent — the live target may be serving broken content", "Restore the previous version manually before retrying the deploy", "Keep an out-of-band record of the last known good artifact", "Check whether credentials expired between deploy and rollback"],
        example_bad: None,
        example_good: None,
        see_also: &["DeployDeployFailed", "DeployAdapterError", "DeployConfigError"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(145),
        long_code: "HHS_E_NAVIGATION_BLOCKED",
        short_code: "E0145",
        title: "Navigation Blocked By Guard",
        short_description: "A navigation request was rejected because a route guard or in-progress transition refused to release control.",
        long_description: "The UI navigator supports guards: predicates that can veto a route change for reasons like unsaved form state or missing authentication. When a guard returns false (or another transition is mid-flight and not yet committed), the new request is rejected with this variant.

No state is mutated when a navigation is blocked. The current screen remains active, and the requesting code receives the error so it can prompt the user, save state, and retry.

Check which guard fired and decide whether to satisfy its precondition or override it. Forcing a navigation past a guard requires an explicit API call and should be rare.",
        hints: &["Identify which guard rejected the navigation", "Save unsaved state and retry rather than forcing the transition", "Avoid issuing back-to-back navigations during a transition", "Use the explicit force API only as a last resort"],
        example_bad: None,
        example_good: None,
        see_also: &["NavigationScreenNotFound", "NavigationNoHistory"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(146),
        long_code: "HHS_E_NAVIGATION_NO_HISTORY",
        short_code: "E0146",
        title: "No Navigation History To Pop",
        short_description: "A back navigation was requested but the history stack is empty, leaving nowhere to return to.",
        long_description: "Back navigation in the UI navigator pops the most recent entry off the history stack. If the stack is empty — typically because the user is on the initial route or because the stack was just cleared — the pop has no destination and this error is returned.

This is an expected condition rather than a bug, but it should still be handled. A common pattern is to translate the error into a no-op or into an explicit \"exit application\" decision based on platform conventions.

Check whether you need to seed the history stack on launch, or guard back calls behind a stack-depth check.",
        hints: &["Guard back navigations with a history-depth check", "Translate the error into a platform-appropriate exit on root screens", "Seed the history stack on launch if your model expects entries", "Avoid clearing history without re-seeding the current route"],
        example_bad: None,
        example_good: None,
        see_also: &["NavigationBlocked", "NavigationScreenNotFound"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    },

    ExceptionEntry {
        code: ExceptionCode(147),
        long_code: "HHS_E_NAVIGATION_SCREEN_NOT_FOUND",
        short_code: "E0147",
        title: "Navigation Target Screen Unknown",
        short_description: "The navigator could not resolve the requested route name to any registered screen.",
        long_description: "Every screen the UI navigator can show must be registered with a name. When a navigate call references a name that is not in the registry, the request fails before any state changes. The cause is typically a typo, a stale string constant, or a screen module that was removed without cleaning up its callers.

The error includes the offending name so you can search for it in your codebase. The current screen is unaffected.

Verify the name against the registration site, fix the caller, and consider extracting route names into typed constants to prevent repeats.",
        hints: &["Compare the failing name against your screen registration list", "Extract route names into typed constants to avoid typos", "Check whether a recent refactor removed the screen module", "Surface a 404-style fallback for user-facing deep links"],
        example_bad: None,
        example_good: None,
        see_also: &["NavigationBlocked", "NavigationNoHistory"],
        since_version: "0.4.47",
        category: ExceptionCategory::Ui,
    }
];
