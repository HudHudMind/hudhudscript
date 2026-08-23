use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
mod late;
mod platform;

impl VM {
    pub(crate) fn register_globals(&mut self) {
        // Default `this` binding — empty object so script-level
        // `this.call({...})` can dispatch through provider dispatch
        // (interpreter parity). Class / instance methods overwrite this
        // with the real receiver at method-call time and restore the
        // previous value on return (T5.2: now stored in VM.cur_this field).

        // Math object with constants
        let mut math_obj = hudhudscript_bytecode::ObjMap::default();
        math_obj.insert("PI".to_string(), Value16::number(std::f64::consts::PI));
        math_obj.insert("E".to_string(), Value16::number(std::f64::consts::E));
        math_obj.insert("__module".to_string(), Value16::string("Math".to_string()));
        let math_val = Value16::object(math_obj);
        self.set_global("Math", math_val);
        self.math_obj = Some(math_val);

        // JSON object (marker for method dispatch)
        let mut json_obj = hudhudscript_bytecode::ObjMap::default();
        json_obj.insert("__module".to_string(), Value16::string("JSON".to_string()));
        let json_val = Value16::object(json_obj);
        self.set_global("JSON", json_val);
        self.json_obj = Some(json_val);

        // None constant
        self.set_global("None", Value16::null());

        // http object (marker for method dispatch)
        let mut http_obj = hudhudscript_bytecode::ObjMap::default();
        http_obj.insert("__module".to_string(), Value16::string("http".to_string()));
        self.set_global("http", Value16::object(http_obj));

        // file object (marker for method dispatch)
        let mut file_obj = hudhudscript_bytecode::ObjMap::default();
        file_obj.insert("__module".to_string(), Value16::string("file".to_string()));
        self.set_global("file", Value16::object(file_obj));

        // Promise object (marker for method dispatch)
        let mut promise_obj = hudhudscript_bytecode::ObjMap::default();
        promise_obj.insert(
            "__module".to_string(),
            Value16::string("Promise".to_string()),
        );
        self.set_global("Promise", Value16::object(promise_obj));

        // MCP object (marker for method dispatch)
        let mut mcp_obj = hudhudscript_bytecode::ObjMap::default();
        mcp_obj.insert("__module".to_string(), Value16::string("mcp".to_string()));
        self.set_global("mcp", Value16::object(mcp_obj));

        // linalg module — interpreter exposed this as `LinAlg`.  Register
        // both spellings for back-compat; the dispatcher key remains the
        // lowercase registration name from `register_vm_stdlib_modules`.
        let mut linalg_obj = hudhudscript_bytecode::ObjMap::default();
        linalg_obj.insert(
            "__module".to_string(),
            Value16::string("linalg".to_string()),
        );
        self.set_global("linalg", Value16::object(linalg_obj.clone()));
        self.set_global("LinAlg", Value16::object(linalg_obj));

        // stats module — interpreter exposed this as `Stats`.  Same alias
        // pattern as `LinAlg` above.
        let mut stats_obj = hudhudscript_bytecode::ObjMap::default();
        stats_obj.insert("__module".to_string(), Value16::string("stats".to_string()));
        self.set_global("stats", Value16::object(stats_obj.clone()));
        self.set_global("Stats", Value16::object(stats_obj));

        // Serialization modules (v0.4.38 — #650)
        let mut toml_obj = hudhudscript_bytecode::ObjMap::default();
        toml_obj.insert("__module".to_string(), Value16::string("TOML".to_string()));
        self.set_global("TOML", Value16::object(toml_obj));

        let mut yaml_obj = hudhudscript_bytecode::ObjMap::default();
        yaml_obj.insert("__module".to_string(), Value16::string("YAML".to_string()));
        self.set_global("YAML", Value16::object(yaml_obj));

        let mut csv_obj = hudhudscript_bytecode::ObjMap::default();
        csv_obj.insert("__module".to_string(), Value16::string("CSV".to_string()));
        self.set_global("CSV", Value16::object(csv_obj));

        let mut ini_obj = hudhudscript_bytecode::ObjMap::default();
        ini_obj.insert("__module".to_string(), Value16::string("INI".to_string()));
        self.set_global("INI", Value16::object(ini_obj));

        // Encoding modules (v0.4.38 — #651)
        let mut base64_obj = hudhudscript_bytecode::ObjMap::default();
        base64_obj.insert(
            "__module".to_string(),
            Value16::string("Base64".to_string()),
        );
        self.set_global("Base64", Value16::object(base64_obj));

        let mut hex_obj = hudhudscript_bytecode::ObjMap::default();
        hex_obj.insert("__module".to_string(), Value16::string("Hex".to_string()));
        self.set_global("Hex", Value16::object(hex_obj));

        let mut url_obj = hudhudscript_bytecode::ObjMap::default();
        url_obj.insert("__module".to_string(), Value16::string("URL".to_string()));
        self.set_global("URL", Value16::object(url_obj));

        // UUID module (v0.4.38 — #652)
        let mut uuid_obj = hudhudscript_bytecode::ObjMap::default();
        uuid_obj.insert("__module".to_string(), Value16::string("uuid".to_string()));
        self.set_global("uuid", Value16::object(uuid_obj));

        // Path module (v0.4.38 — #663)
        let mut path_obj = hudhudscript_bytecode::ObjMap::default();
        path_obj.insert("__module".to_string(), Value16::string("Path".to_string()));
        self.set_global("Path", Value16::object(path_obj));

        // Temp module (v0.4.38 — #664)
        let mut temp_obj = hudhudscript_bytecode::ObjMap::default();
        temp_obj.insert("__module".to_string(), Value16::string("Temp".to_string()));
        self.set_global("Temp", Value16::object(temp_obj));

        // URLParser module (v0.4.38 — #665)
        let mut urlparser_obj = hudhudscript_bytecode::ObjMap::default();
        urlparser_obj.insert(
            "__module".to_string(),
            Value16::string("URLParser".to_string()),
        );
        self.set_global("URLParser", Value16::object(urlparser_obj));

        // Glob module (v0.4.38 — #666)
        let mut glob_obj = hudhudscript_bytecode::ObjMap::default();
        glob_obj.insert("__module".to_string(), Value16::string("Glob".to_string()));
        self.set_global("Glob", Value16::object(glob_obj));

        // Set module (v0.4.38 — #653)
        let mut set_obj = hudhudscript_bytecode::ObjMap::default();
        set_obj.insert("__module".to_string(), Value16::string("Set".to_string()));
        self.set_global("Set", Value16::object(set_obj));

        // Map module (v0.4.38 — #654)
        let mut map_obj = hudhudscript_bytecode::ObjMap::default();
        map_obj.insert("__module".to_string(), Value16::string("Map".to_string()));
        self.set_global("Map", Value16::object(map_obj));

        // stdin module (v0.4.38 — #656)
        let mut stdin_obj = hudhudscript_bytecode::ObjMap::default();
        stdin_obj.insert("__module".to_string(), Value16::string("stdin".to_string()));
        self.set_global("stdin", Value16::object(stdin_obj));

        // Terminal module (v0.4.38 — #657)
        let mut terminal_obj = hudhudscript_bytecode::ObjMap::default();
        terminal_obj.insert(
            "__module".to_string(),
            Value16::string("Terminal".to_string()),
        );
        self.set_global("Terminal", Value16::object(terminal_obj));

        // log module (v0.4.38 — #662)
        let mut log_obj = hudhudscript_bytecode::ObjMap::default();
        log_obj.insert("__module".to_string(), Value16::string("log".to_string()));
        self.set_global("log", Value16::object(log_obj));

        // exec module (v0.4.38 — #674)
        let mut exec_obj = hudhudscript_bytecode::ObjMap::default();
        exec_obj.insert("__module".to_string(), Value16::string("exec".to_string()));
        self.set_global("exec", Value16::object(exec_obj));

        // TCP module (v0.4.38 — #675)
        let mut tcp_obj = hudhudscript_bytecode::ObjMap::default();
        tcp_obj.insert("__module".to_string(), Value16::string("tcp".to_string()));
        self.set_global("tcp", Value16::object(tcp_obj));

        // UDP module (v0.4.38 — #675)
        let mut udp_obj = hudhudscript_bytecode::ObjMap::default();
        udp_obj.insert("__module".to_string(), Value16::string("udp".to_string()));
        self.set_global("udp", Value16::object(udp_obj));

        // Unix domain socket module (v0.4.38 — #676)
        let mut unix_obj = hudhudscript_bytecode::ObjMap::default();
        unix_obj.insert("__module".to_string(), Value16::string("unix".to_string()));
        self.set_global("unix", Value16::object(unix_obj));

        // WebSocket module (v0.4.38 — #616)
        let mut ws_obj = hudhudscript_bytecode::ObjMap::default();
        ws_obj.insert("__module".to_string(), Value16::string("ws".to_string()));
        self.set_global("ws", Value16::object(ws_obj));

        // Daemon/Service module (v0.4.38 — #596)
        let mut daemon_obj = hudhudscript_bytecode::ObjMap::default();
        daemon_obj.insert(
            "__module".to_string(),
            Value16::string("daemon".to_string()),
        );
        self.set_global("daemon", Value16::object(daemon_obj));

        // Filesystem operations module (v0.4.38 — #604)
        let mut fs_obj = hudhudscript_bytecode::ObjMap::default();
        fs_obj.insert("__module".to_string(), Value16::string("fs".to_string()));
        self.set_global("fs", Value16::object(fs_obj));

        // Environment module (v0.4.38 — #622)
        let mut env_mod_obj = hudhudscript_bytecode::ObjMap::default();
        env_mod_obj.insert("__module".to_string(), Value16::string("Env".to_string()));
        self.set_global("Env", Value16::object(env_mod_obj));

        // Tokenomics module (T3 — #TOK-3)
        let mut tok_obj = hudhudscript_bytecode::ObjMap::default();
        tok_obj.insert(
            "__module".to_string(),
            Value16::string("tokenomics".to_string()),
        );
        self.set_global("tokenomics", Value16::object(tok_obj));

        // Channel module (CH2 — #CH-2)
        let mut chan_obj = hudhudscript_bytecode::ObjMap::default();
        chan_obj.insert(
            "__module".to_string(),
            Value16::string("channel".to_string()),
        );
        self.set_global("channel", Value16::object(chan_obj));

        // OS info module (v0.4.38 — #622)
        let mut os_obj = hudhudscript_bytecode::ObjMap::default();
        os_obj.insert("__module".to_string(), Value16::string("os".to_string()));
        self.set_global("os", Value16::object(os_obj));

        // Date/Time module (v0.4.38 — #593)
        let mut date_obj = hudhudscript_bytecode::ObjMap::default();
        date_obj.insert("__module".to_string(), Value16::string("Date".to_string()));
        self.set_global("Date", Value16::object(date_obj));

        // Duration module (v0.4.38 — #593)
        let mut duration_obj = hudhudscript_bytecode::ObjMap::default();
        duration_obj.insert(
            "__module".to_string(),
            Value16::string("Duration".to_string()),
        );
        self.set_global("Duration", Value16::object(duration_obj));

        // Regex module (v0.4.38 — #592)
        let mut regex_obj = hudhudscript_bytecode::ObjMap::default();
        regex_obj.insert("__module".to_string(), Value16::string("regex".to_string()));
        self.set_global("regex", Value16::object(regex_obj));

        // Schedule module (v0.4.38 — #618)
        let mut sched_obj = hudhudscript_bytecode::ObjMap::default();
        sched_obj.insert(
            "__module".to_string(),
            Value16::string("schedule".to_string()),
        );
        self.set_global("schedule", Value16::object(sched_obj));

        // EventBus / IPC module (v0.4.38 — #597)
        let mut event_bus_obj = hudhudscript_bytecode::ObjMap::default();
        event_bus_obj.insert(
            "__module".to_string(),
            Value16::string("EventBus".to_string()),
        );
        self.set_global("EventBus", Value16::object(event_bus_obj));

        // Plugin lifecycle module (v0.4.38 — #598)
        let mut plugin_obj = hudhudscript_bytecode::ObjMap::default();
        plugin_obj.insert(
            "__module".to_string(),
            Value16::string("Plugin".to_string()),
        );
        self.set_global("Plugin", Value16::object(plugin_obj));

        // MCP Server module (v0.4.38 — #600)
        let mut mcp_server_obj = hudhudscript_bytecode::ObjMap::default();
        mcp_server_obj.insert(
            "__module".to_string(),
            Value16::string("McpServer".to_string()),
        );
        self.set_global("McpServer", Value16::object(mcp_server_obj));

        // HTTP Server module (v0.4.38 — #602)
        let mut server_obj = hudhudscript_bytecode::ObjMap::default();
        server_obj.insert(
            "__module".to_string(),
            Value16::string("Server".to_string()),
        );
        self.set_global("Server", Value16::object(server_obj));

        // Per-plugin config module (v0.4.38 — #610)
        let mut plugin_config_obj = hudhudscript_bytecode::ObjMap::default();
        plugin_config_obj.insert(
            "__module".to_string(),
            Value16::string("PluginConfig".to_string()),
        );
        self.set_global("PluginConfig", Value16::object(plugin_config_obj));

        // Cryptography module (v0.4.38 — #614) — SHA-2, BLAKE3, HMAC,
        // AES-256-GCM, Argon2id, secure RNG. Dispatched via shared
        // crypto_ops (Kural 7). See hudhudscript-cli's
        // register_vm_stdlib_modules for the method handler.
        let mut crypto_obj = hudhudscript_bytecode::ObjMap::default();
        crypto_obj.insert(
            "__module".to_string(),
            Value16::string("crypto".to_string()),
        );
        self.set_global("crypto", Value16::object(crypto_obj));

        // Archive / compression module (v0.4.38 — #589) — tar.gz, zip,
        // gzip, deflate. Dispatched via shared archive_ops (Kural 7).
        let mut archive_obj = hudhudscript_bytecode::ObjMap::default();
        archive_obj.insert(
            "__module".to_string(),
            Value16::string("archive".to_string()),
        );
        self.set_global("archive", Value16::object(archive_obj));

        // System metrics module (v0.4.38 — #588) — CPU/memory/disk/load,
        // uptime, hostname, net interfaces, processes. Shared dispatch
        // (Kural 7) via hudhudscript_builtins::system_metrics_ops.
        let mut system_obj = hudhudscript_bytecode::ObjMap::default();
        system_obj.insert(
            "__module".to_string(),
            Value16::string("system".to_string()),
        );
        self.set_global("system", Value16::object(system_obj));

        // Download manager (v0.4.38 — #590) — HTTP GET/HEAD, resume,
        // progress, text/json. Shared dispatch (Kural 7).
        let mut download_obj = hudhudscript_bytecode::ObjMap::default();
        download_obj.insert(
            "__module".to_string(),
            Value16::string("download".to_string()),
        );
        self.set_global("download", Value16::object(download_obj));

        // Docker CLI wrapper (v0.4.38 — #620) — ps, images, run, stop,
        // rm, logs, exec, build. Shared dispatch (Kural 7).
        let mut docker_obj = hudhudscript_bytecode::ObjMap::default();
        docker_obj.insert(
            "__module".to_string(),
            Value16::string("docker".to_string()),
        );
        self.set_global("docker", Value16::object(docker_obj));

        // Email / messaging (v0.4.38 — #619) — SMTP, MIME parse,
        // Maildir, Telegram, webhook POST. Shared dispatch (Kural 7).
        let mut email_obj = hudhudscript_bytecode::ObjMap::default();
        email_obj.insert("__module".to_string(), Value16::string("email".to_string()));
        self.set_global("email", Value16::object(email_obj));

        // Security scanning (v0.4.38 — #636) — SUID, SSL, world-writable,
        // open ports, failed logins, permissions. Shared (Kural 7).
        let mut security_obj = hudhudscript_bytecode::ObjMap::default();
        security_obj.insert(
            "__module".to_string(),
            Value16::string("security".to_string()),
        );
        self.set_global("security", Value16::object(security_obj));

        // Desktop notification / systemd journal (v0.4.38 — #611). Shared (Kural 7).
        let mut notify_obj = hudhudscript_bytecode::ObjMap::default();
        notify_obj.insert(
            "__module".to_string(),
            Value16::string("notify".to_string()),
        );
        self.set_global("notify", Value16::object(notify_obj));

        // UFW firewall (v0.4.38 — #645). Shared dispatch (Kural 7).
        let mut firewall_obj = hudhudscript_bytecode::ObjMap::default();
        firewall_obj.insert(
            "__module".to_string(),
            Value16::string("firewall".to_string()),
        );
        self.set_global("firewall", Value16::object(firewall_obj));

        // Unix-domain socket (v0.4.38 — #676). Shared (Kural 7).
        let mut unix_obj = hudhudscript_bytecode::ObjMap::default();
        unix_obj.insert("__module".to_string(), Value16::string("unix".to_string()));
        self.set_global("unix", Value16::object(unix_obj));

        // APT package manager (v0.4.38 — #635). Shared (Kural 7).
        let mut apt_obj = hudhudscript_bytecode::ObjMap::default();
        apt_obj.insert("__module".to_string(), Value16::string("apt".to_string()));
        self.set_global("apt", Value16::object(apt_obj));

        // XDG desktop integration (v0.4.38 — #591). Shared (Kural 7).
        let mut xdg_obj = hudhudscript_bytecode::ObjMap::default();
        xdg_obj.insert("__module".to_string(), Value16::string("xdg".to_string()));
        self.set_global("xdg", Value16::object(xdg_obj));

        // Plugin code signing (v0.4.38 — #648). Shared (Kural 7).
        let mut codesign_obj = hudhudscript_bytecode::ObjMap::default();
        codesign_obj.insert(
            "__module".to_string(),
            Value16::string("codesign".to_string()),
        );
        self.set_global("codesign", Value16::object(codesign_obj));

        self.register_late_globals();
        self.register_platform_globals();
    }
}
