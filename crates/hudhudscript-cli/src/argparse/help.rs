use super::*;
impl ArgParser {
    /// Generate a `--help` text string.
    pub fn generate_help(&self) -> String {
        let mut out = String::new();

        // Header
        if !self.description.is_empty() {
            out.push_str(&self.description);
            out.push('\n');
        }
        if !self.version.is_empty() {
            out.push_str(&format!("{} {}\n", self.program_name, self.version));
        }
        out.push('\n');

        // Usage
        out.push_str(&format!("USAGE:\n    {}", self.program_name));
        if !self.subcommands.is_empty() {
            out.push_str(" <COMMAND>");
        }
        if !self.args.is_empty() {
            out.push_str(" [OPTIONS]");
        }
        out.push('\n');

        // Subcommands
        if !self.subcommands.is_empty() {
            out.push_str("\nCOMMANDS:\n");
            let max_width = self
                .subcommands
                .iter()
                .map(|s| s.name.len())
                .max()
                .unwrap_or(0);
            for sc in &self.subcommands {
                out.push_str(&format!(
                    "    {:<width$}    {}\n",
                    sc.name,
                    sc.description,
                    width = max_width
                ));
            }
        }

        // Options
        if !self.args.is_empty() {
            out.push_str("\nOPTIONS:\n");
            Self::format_args_help(&self.args, &mut out);
        }

        // Built-in flags
        out.push_str("\n    -h, --help       Print help information\n");
        out.push_str("    -V, --version    Print version information\n");

        out
    }

    /// Generate help section for a subcommand.
    pub fn generate_subcommand_help(&self, name: &str) -> Option<String> {
        let sc = self.subcommands.iter().find(|s| s.name == name)?;
        let mut out = String::new();
        out.push_str(&format!("{} {}\n", self.program_name, sc.name));
        if !sc.description.is_empty() {
            out.push_str(&sc.description);
            out.push('\n');
        }
        out.push_str(&format!(
            "\nUSAGE:\n    {} {} [OPTIONS]\n",
            self.program_name, sc.name
        ));

        if !sc.args.is_empty() {
            out.push_str("\nOPTIONS:\n");
            Self::format_args_help(&sc.args, &mut out);
        }
        out.push_str("\n    -h, --help    Print help information\n");
        Some(out)
    }

    fn format_args_help(args: &[Arg], out: &mut String) {
        let max_width = args
            .iter()
            .map(|a| a.flags_display().len())
            .max()
            .unwrap_or(0);
        for arg in args {
            let flags = arg.flags_display();
            let mut line = format!(
                "    {:<width$}    {}",
                flags,
                arg.description,
                width = max_width
            );
            if arg.required {
                line.push_str(" [required]");
            }
            if let Some(ref dv) = arg.default_value {
                line.push_str(&format!(" [default: {}]", dv));
            }
            out.push_str(&line);
            out.push('\n');
        }
    }

    // ── Completion generation ────────────────────────────────────────────

    /// Generate a shell completion script for the given shell.
    pub fn generate_completion(&self, shell: Shell) -> String {
        match shell {
            Shell::Bash => self.generate_bash_completion(),
            Shell::Zsh => self.generate_zsh_completion(),
            Shell::Fish => self.generate_fish_completion(),
        }
    }

    fn generate_bash_completion(&self) -> String {
        let name = &self.program_name;
        let mut out = format!(
            "# Bash completion for {name}\n\
             _{name}_completions() {{\n\
             \x20   local cur prev opts cmds\n\
             \x20   COMPREPLY=()\n\
             \x20   cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\
             \x20   prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n"
        );

        // Subcommands
        if !self.subcommands.is_empty() {
            let cmds: Vec<&str> = self.subcommands.iter().map(|s| s.name.as_str()).collect();
            out.push_str(&format!("    cmds=\"{}\"\n", cmds.join(" ")));
        }

        // Top-level options
        let opts = self.collect_flags(&self.args);
        out.push_str(&format!("    opts=\"{}\"\n", opts.join(" ")));

        out.push_str(
            "    if [[ ${COMP_CWORD} -eq 1 ]]; then\n\
             \x20       COMPREPLY=( $(compgen -W \"${cmds} ${opts}\" -- \"${cur}\") )\n\
             \x20       return 0\n\
             \x20   fi\n",
        );

        // Per-subcommand completions
        for sc in &self.subcommands {
            let sc_opts = self.collect_flags(&sc.args);
            out.push_str(&format!(
                "    if [[ \"${{COMP_WORDS[1]}}\" == \"{}\" ]]; then\n\
                 \x20       COMPREPLY=( $(compgen -W \"{}\" -- \"${{cur}}\") )\n\
                 \x20       return 0\n\
                 \x20   fi\n",
                sc.name,
                sc_opts.join(" ")
            ));
        }

        out.push_str("}\n");
        out.push_str(&format!("complete -F _{name}_completions {name}\n"));
        out
    }

    fn generate_zsh_completion(&self) -> String {
        let name = &self.program_name;
        let mut out = format!("#compdef {name}\n\n_{name}() {{\n");

        if !self.subcommands.is_empty() {
            out.push_str("    local -a subcmds\n    subcmds=(\n");
            for sc in &self.subcommands {
                out.push_str(&format!(
                    "        '{}:{}'\n",
                    sc.name,
                    sc.description.replace('\'', "'\\''")
                ));
            }
            out.push_str("    )\n\n");
        }

        out.push_str("    _arguments -s \\\n");
        for arg in &self.args {
            if let Some(ref l) = arg.long {
                let desc = arg.description.replace('\'', "'\\''");
                if arg.arg_type == ArgType::Bool {
                    out.push_str(&format!("        '--{l}[{desc}]' \\\n"));
                } else {
                    out.push_str(&format!("        '--{l}=[{desc}]:value:' \\\n"));
                }
            }
        }

        if !self.subcommands.is_empty() {
            out.push_str("        '1:command:->cmds' \\\n");
            out.push_str("        '*::arg:->args'\n\n");
            out.push_str("    case $state in\n");
            out.push_str("        cmds) _describe 'command' subcmds ;;\n");
            out.push_str("    esac\n");
        } else {
            // Remove trailing backslash-newline
            if out.ends_with(" \\\n") {
                out.truncate(out.len() - 3);
                out.push('\n');
            }
        }

        out.push_str("}\n\n");
        out.push_str(&format!("_{name} \"$@\"\n"));
        out
    }

    fn generate_fish_completion(&self) -> String {
        let name = &self.program_name;
        let mut out = format!("# Fish completion for {name}\n");

        // Subcommands
        for sc in &self.subcommands {
            out.push_str(&format!(
                "complete -c {name} -n '__fish_use_subcommand' -a '{}' -d '{}'\n",
                sc.name,
                sc.description.replace('\'', "'\\''")
            ));
        }

        // Top-level args
        for arg in &self.args {
            if let Some(ref l) = arg.long {
                let desc = arg.description.replace('\'', "'\\''");
                let mut cmd = format!("complete -c {name} -l '{l}'");
                if let Some(s) = arg.short {
                    cmd.push_str(&format!(" -s '{s}'"));
                }
                cmd.push_str(&format!(" -d '{desc}'"));
                if arg.arg_type != ArgType::Bool {
                    cmd.push_str(" -r"); // requires argument
                }
                out.push_str(&cmd);
                out.push('\n');
            }
        }

        // Subcommand args
        for sc in &self.subcommands {
            for arg in &sc.args {
                if let Some(ref l) = arg.long {
                    let desc = arg.description.replace('\'', "'\\''");
                    let mut cmd = format!(
                        "complete -c {name} -n '__fish_seen_subcommand_from {}' -l '{l}'",
                        sc.name
                    );
                    if let Some(s) = arg.short {
                        cmd.push_str(&format!(" -s '{s}'"));
                    }
                    cmd.push_str(&format!(" -d '{desc}'"));
                    if arg.arg_type != ArgType::Bool {
                        cmd.push_str(" -r");
                    }
                    out.push_str(&cmd);
                    out.push('\n');
                }
            }
        }

        out
    }

    /// Collect all long/short flags from a slice of `Arg` definitions.
    fn collect_flags(&self, args: &[Arg]) -> Vec<String> {
        let mut flags = Vec::new();
        for arg in args {
            if let Some(ref l) = arg.long {
                flags.push(format!("--{}", l));
            }
            if let Some(s) = arg.short {
                flags.push(format!("-{}", s));
            }
        }
        flags.push("--help".to_string());
        flags
    }
}
