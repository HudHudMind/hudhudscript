use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

impl VM {
    pub(crate) fn register_platform_globals(&mut self) {
        // PDF / poppler (v0.4.38 — #642). Shared (Kural 7).
        let mut pdf_obj = HashMap::new();
        pdf_obj.insert("__module".to_string(), Value16::string("pdf".to_string()));
        self.set_global("pdf", Value16::object(pdf_obj));

        // D-Bus (v0.4.38 — #613). Shared (Kural 7).
        let mut dbus_obj = HashMap::new();
        dbus_obj.insert("__module".to_string(), Value16::string("dbus".to_string()));
        self.set_global("dbus", Value16::object(dbus_obj));

        // GPU (v0.4.38 — #615). Shared (Kural 7).
        let mut gpu_obj = HashMap::new();
        gpu_obj.insert("__module".to_string(), Value16::string("gpu".to_string()));
        self.set_global("gpu", Value16::object(gpu_obj));

        // LibreTranslate (v0.4.38 — #641). Shared (Kural 7).
        let mut translate_obj = HashMap::new();
        translate_obj.insert(
            "__module".to_string(),
            Value16::string("translate".to_string()),
        );
        self.set_global("translate", Value16::object(translate_obj));

        // E2E encryption (Issue #647). Shared (Kural 7).
        let mut e2e_obj = HashMap::new();
        e2e_obj.insert("__module".to_string(), Value16::string("e2e".to_string()));
        self.set_global("e2e", Value16::object(e2e_obj));

        // Hardware detection (v0.4.38 — #629). Shared (Kural 7).
        let mut hardware_obj = HashMap::new();
        hardware_obj.insert(
            "__module".to_string(),
            Value16::string("hardware".to_string()),
        );
        self.set_global("hardware", Value16::object(hardware_obj));

        // Project environment detection (v0.4.38 — #637). Shared (Kural 7).
        let mut project_obj = HashMap::new();
        project_obj.insert(
            "__module".to_string(),
            Value16::string("project".to_string()),
        );
        self.set_global("project", Value16::object(project_obj));

        // Media (Issue #617). Shared (Kural 7).
        let mut media_obj = HashMap::new();
        media_obj.insert("__module".to_string(), Value16::string("media".to_string()));
        self.set_global("media", Value16::object(media_obj));

        // Transmission RPC (v0.4.38 — #639). Shared (Kural 7).
        let mut torrent_obj = HashMap::new();
        torrent_obj.insert(
            "__module".to_string(),
            Value16::string("torrent".to_string()),
        );
        self.set_global("torrent", Value16::object(torrent_obj));

        // MPRIS media player control (v0.4.38 — #638). Shared (Kural 7).
        let mut mpris_obj = HashMap::new();
        mpris_obj.insert("__module".to_string(), Value16::string("mpris".to_string()));
        self.set_global("mpris", Value16::object(mpris_obj));

        // Text-to-Speech (espeak-ng / piper / festival). Shared (Kural 7).
        let mut tts_obj = HashMap::new();
        tts_obj.insert("__module".to_string(), Value16::string("tts".to_string()));
        self.set_global("tts", Value16::object(tts_obj));

        // Browser integration (v0.4.38 — #643). Shared (Kural 7).
        let mut browser_obj = HashMap::new();
        browser_obj.insert(
            "__module".to_string(),
            Value16::string("browser".to_string()),
        );
        self.set_global("browser", Value16::object(browser_obj));

        // Error base class (v0.4.38 — #669)
        let mut error_obj = HashMap::new();
        error_obj.insert("__class".to_string(), Value16::string("Error".to_string()));
        error_obj.insert("name".to_string(), Value16::string("Error".to_string()));
        error_obj.insert("message".to_string(), Value16::string(String::new()));
        error_obj.insert("stack".to_string(), Value16::string(String::new()));
        self.set_global("Error", Value16::object(error_obj));

        // `env` global — snapshot of the process environment.  Interpreter
        // exposed this via init_globals (deleted in the interpreter-crate
        // retirement).  Snapshot at VM construction so scripts like
        // `env.HOME`, `env.PATH` resolve through the normal GetProperty
        // path on Value::Object.  A snapshot (not a proxy) matches how the
        // interpreter behaved — later `std::env::set_var` calls from native
        // ops are not reflected in the user-visible `env` unless the
        // script re-reads via Env.get / os.env etc.
        //
        // Gap 3 (interpreter parity): tag the snapshot with the
        // `__hudhud_env` marker so `GetProperty` can tell it apart from
        // user-authored objects and return an empty string for missing
        // keys instead of the generic "Property not found" runtime error.
        // Mirrors the interpreter's `env_lookup` which fell back to
        // `Ok(Value16::string(String::new()))` on miss.
        let mut env_obj: HashMap<String, Value16> = HashMap::new();
        for (k, v) in std::env::vars() {
            env_obj.insert(k, Value16::string(v.to_string()));
        }
        env_obj.insert("__hudhud_env".to_string(), Value16::bool_(true));
        self.set_global("env", Value16::object(env_obj));
    }
}
