use super::*;
impl ArgParser {
    /// Create a new parser.
    pub fn new(program_name: impl Into<String>) -> Self {
        Self {
            program_name: program_name.into(),
            description: String::new(),
            version: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    /// Set the program description.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }

    /// Set the program version.
    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.version = v.into();
        self
    }

    /// Add a top-level argument.
    pub fn arg(mut self, a: Arg) -> Self {
        self.args.push(a);
        self
    }

    /// Add a subcommand.
    pub fn subcommand(mut self, sc: Subcommand) -> Self {
        self.subcommands.push(sc);
        self
    }

    // ── Parsing ──────────────────────────────────────────────────────────

    /// Parse the given argument slice.
    pub fn parse(&self, args: &[String]) -> ArgResult<ParsedArgs> {
        // Check for --help / --version first
        for a in args {
            if a == "--help" || a == "-h" {
                return Err(ArgError::Other(self.generate_help()));
            }
            if a == "--version" || a == "-V" {
                return Err(ArgError::Other(format!(
                    "{} {}",
                    self.program_name, self.version
                )));
            }
        }

        // Check if the first arg matches a subcommand
        if let Some(first) = args.first() {
            if let Some(sc) = self.subcommands.iter().find(|s| s.name == *first) {
                let sub_parsed = Self::parse_args_against(&sc.args, &args[1..])?;
                let mut result = ParsedArgs::new();
                result.subcommand = Some((sc.name.clone(), Box::new(sub_parsed)));
                return Ok(result);
            }
        }

        // Parse against top-level args
        Self::parse_args_against(&self.args, args)
    }

    /// Core parsing logic shared between top-level and subcommand contexts.
    fn parse_args_against(defs: &[Arg], tokens: &[String]) -> ArgResult<ParsedArgs> {
        let mut parsed = ParsedArgs::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            if let Some(key) = token.strip_prefix("--") {
                if let Some(def) = defs.iter().find(|d| d.long.as_deref() == Some(key)) {
                    i = Self::consume_value(def, tokens, i, &mut parsed)?;
                } else {
                    return Err(ArgError::UnknownArgument(token.clone()));
                }
            } else if token.starts_with('-') && token.len() == 2 {
                let ch = token.chars().nth(1).unwrap();
                if let Some(def) = defs.iter().find(|d| d.short == Some(ch)) {
                    i = Self::consume_value(def, tokens, i, &mut parsed)?;
                } else {
                    return Err(ArgError::UnknownArgument(token.clone()));
                }
            } else {
                // Positional
                parsed.positional.push(token.clone());
                i += 1;
            }
        }

        // Apply defaults and check required
        for def in defs {
            if !parsed.has(&def.name)
                && !parsed.positional.is_empty()
                && def.arg_type != ArgType::Bool
            {
                // skip -- positionals are handled separately
            }
            if !parsed.has(&def.name) {
                if let Some(ref default) = def.default_value {
                    if def.arg_type == ArgType::List {
                        let items: Vec<String> =
                            default.split(',').map(|s| s.trim().to_string()).collect();
                        parsed.list_values.insert(def.name.clone(), items);
                    } else {
                        parsed.values.insert(def.name.clone(), default.clone());
                    }
                } else if def.required {
                    return Err(ArgError::MissingRequired(def.name.clone()));
                }
            }
        }

        Ok(parsed)
    }

    /// Consume a value for the given arg definition starting at index `i`.
    /// Returns the next index to process.
    fn consume_value(
        def: &Arg,
        tokens: &[String],
        i: usize,
        parsed: &mut ParsedArgs,
    ) -> ArgResult<usize> {
        match def.arg_type {
            ArgType::Bool => {
                parsed.values.insert(def.name.clone(), "true".to_string());
                Ok(i + 1)
            }
            ArgType::List => {
                if i + 1 >= tokens.len() {
                    return Err(ArgError::InvalidValue {
                        arg: def.name.clone(),
                        expected: def.arg_type,
                        got: String::new(),
                    });
                }
                let raw = &tokens[i + 1];
                let items: Vec<String> = raw.split(',').map(|s| s.trim().to_string()).collect();
                parsed.list_values.insert(def.name.clone(), items);
                Ok(i + 2)
            }
            ArgType::Int => {
                if i + 1 >= tokens.len() {
                    return Err(ArgError::InvalidValue {
                        arg: def.name.clone(),
                        expected: def.arg_type,
                        got: String::new(),
                    });
                }
                let raw = &tokens[i + 1];
                raw.parse::<i64>().map_err(|_| ArgError::InvalidValue {
                    arg: def.name.clone(),
                    expected: def.arg_type,
                    got: raw.clone(),
                })?;
                parsed.values.insert(def.name.clone(), raw.clone());
                Ok(i + 2)
            }
            ArgType::Float => {
                if i + 1 >= tokens.len() {
                    return Err(ArgError::InvalidValue {
                        arg: def.name.clone(),
                        expected: def.arg_type,
                        got: String::new(),
                    });
                }
                let raw = &tokens[i + 1];
                raw.parse::<f64>().map_err(|_| ArgError::InvalidValue {
                    arg: def.name.clone(),
                    expected: def.arg_type,
                    got: raw.clone(),
                })?;
                parsed.values.insert(def.name.clone(), raw.clone());
                Ok(i + 2)
            }
            ArgType::String => {
                if i + 1 >= tokens.len() {
                    return Err(ArgError::InvalidValue {
                        arg: def.name.clone(),
                        expected: def.arg_type,
                        got: String::new(),
                    });
                }
                parsed
                    .values
                    .insert(def.name.clone(), tokens[i + 1].clone());
                Ok(i + 2)
            }
        }
    }
}
