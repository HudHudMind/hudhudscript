//! Tests for hudhudscript-cli argparse module
//!
//! Covers: ArgParser, Arg, ArgType, ParsedArgs, Subcommand, Shell, ArgError

use hudhudscript_cli::argparse::*;

// ─── Helper ─────────────────────────────────────────────────────────────────

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

// ─── ArgType Display ────────────────────────────────────────────────────────

#[test]
fn argtype_display_all_variants() {
    assert_eq!(format!("{}", ArgType::String), "string");
    assert_eq!(format!("{}", ArgType::Int), "int");
    assert_eq!(format!("{}", ArgType::Float), "float");
    assert_eq!(format!("{}", ArgType::Bool), "bool");
    assert_eq!(format!("{}", ArgType::List), "list");
}

// ─── ArgError Display ───────────────────────────────────────────────────────

#[test]
fn argerror_display_variants() {
    let missing = ArgError::MissingRequired("port".into());
    assert!(missing.to_string().contains("port"));
    assert!(missing.to_string().contains("required"));

    let unknown = ArgError::UnknownArgument("--foo".into());
    assert!(unknown.to_string().contains("--foo"));

    let invalid = ArgError::InvalidValue {
        arg: "count".into(),
        expected: ArgType::Int,
        got: "abc".into(),
    };
    let msg = invalid.to_string();
    assert!(msg.contains("count"));
    assert!(msg.contains("abc"));

    let sub = ArgError::MissingSubcommand;
    assert!(sub.to_string().contains("subcommand"));

    let other = ArgError::Other("custom error".into());
    assert_eq!(other.to_string(), "custom error");
}

// ─── Shell Display ──────────────────────────────────────────────────────────

#[test]
fn shell_display() {
    assert_eq!(format!("{}", Shell::Bash), "bash");
    assert_eq!(format!("{}", Shell::Zsh), "zsh");
    assert_eq!(format!("{}", Shell::Fish), "fish");
}

// ─── Basic flag parsing ─────────────────────────────────────────────────────

#[test]
fn parse_bool_flag_long() {
    let parser = ArgParser::new("test").arg(
        Arg::new("verbose", ArgType::Bool)
            .long("verbose")
            .short('v'),
    );

    let result = parser.parse(&args(&["--verbose"])).unwrap();
    assert_eq!(result.get_bool("verbose"), Some(true));
    assert!(result.has("verbose"));
}

#[test]
fn parse_bool_flag_short() {
    let parser = ArgParser::new("test").arg(
        Arg::new("verbose", ArgType::Bool)
            .long("verbose")
            .short('v'),
    );

    let result = parser.parse(&args(&["-v"])).unwrap();
    assert_eq!(result.get_bool("verbose"), Some(true));
}

// ─── String argument ────────────────────────────────────────────────────────

#[test]
fn parse_string_argument() {
    let parser =
        ArgParser::new("test").arg(Arg::new("name", ArgType::String).long("name").short('n'));

    let result = parser.parse(&args(&["--name", "alice"])).unwrap();
    assert_eq!(result.get_string("name"), Some("alice"));
}

// ─── Int argument (valid + invalid) ─────────────────────────────────────────

#[test]
fn parse_int_argument_valid() {
    let parser = ArgParser::new("test").arg(Arg::new("port", ArgType::Int).long("port").short('p'));

    let result = parser.parse(&args(&["--port", "8080"])).unwrap();
    assert_eq!(result.get_int("port"), Some(8080));
}

#[test]
fn parse_int_argument_invalid_returns_error() {
    let parser = ArgParser::new("test").arg(Arg::new("port", ArgType::Int).long("port"));

    let err = parser.parse(&args(&["--port", "abc"])).unwrap_err();
    match err {
        ArgError::InvalidValue { arg, expected, got } => {
            assert_eq!(arg, "port");
            assert_eq!(expected, ArgType::Int);
            assert_eq!(got, "abc");
        }
        other => panic!("expected InvalidValue, got {:?}", other),
    }
}

// ─── Float argument ─────────────────────────────────────────────────────────

#[test]
fn parse_float_argument() {
    let parser = ArgParser::new("test").arg(Arg::new("rate", ArgType::Float).long("rate"));

    let result = parser.parse(&args(&["--rate", "3.14"])).unwrap();
    let val = result.get_float("rate").unwrap();
    assert!((val - 3.14).abs() < f64::EPSILON);
}

// ─── List argument ──────────────────────────────────────────────────────────

#[test]
fn parse_list_argument() {
    let parser = ArgParser::new("test").arg(Arg::new("tags", ArgType::List).long("tags"));

    let result = parser.parse(&args(&["--tags", "a,b,c"])).unwrap();
    let list = result.get_list("tags").unwrap();
    assert_eq!(list, &["a", "b", "c"]);
    assert!(result.has("tags"));
}

// ─── Required argument missing ──────────────────────────────────────────────

#[test]
fn missing_required_arg_returns_error() {
    let parser = ArgParser::new("test").arg(
        Arg::new("input", ArgType::String)
            .long("input")
            .required(true),
    );

    let err = parser.parse(&args(&[])).unwrap_err();
    match err {
        ArgError::MissingRequired(name) => assert_eq!(name, "input"),
        other => panic!("expected MissingRequired, got {:?}", other),
    }
}

// ─── Default values ─────────────────────────────────────────────────────────

#[test]
fn default_value_used_when_arg_absent() {
    let parser = ArgParser::new("test").arg(
        Arg::new("port", ArgType::Int)
            .long("port")
            .default_value("3000"),
    );

    let result = parser.parse(&args(&[])).unwrap();
    assert_eq!(result.get_int("port"), Some(3000));
}

// ─── Unknown argument ───────────────────────────────────────────────────────

#[test]
fn unknown_long_flag_returns_error() {
    let parser = ArgParser::new("test");
    let err = parser.parse(&args(&["--unknown"])).unwrap_err();
    match err {
        ArgError::UnknownArgument(name) => assert_eq!(name, "--unknown"),
        other => panic!("expected UnknownArgument, got {:?}", other),
    }
}

// ─── Positional arguments ───────────────────────────────────────────────────

#[test]
fn positional_args_collected() {
    let parser = ArgParser::new("test").arg(Arg::new("verbose", ArgType::Bool).long("verbose"));

    let result = parser
        .parse(&args(&["file1.hud", "--verbose", "file2.hud"]))
        .unwrap();
    assert_eq!(result.positional(), &["file1.hud", "file2.hud"]);
    assert_eq!(result.get_bool("verbose"), Some(true));
}

// ─── Subcommand parsing ────────────────────────────────────────────────────

#[test]
fn subcommand_parsing() {
    let parser = ArgParser::new("hud").subcommand(
        Subcommand::new("build", "Build the project")
            .arg(
                Arg::new("release", ArgType::Bool)
                    .long("release")
                    .short('r'),
            )
            .arg(Arg::new("target", ArgType::String).long("target")),
    );

    let result = parser
        .parse(&args(&["build", "--release", "--target", "wasm"]))
        .unwrap();
    let (name, sub_args) = result.subcommand().unwrap();
    assert_eq!(name, "build");
    assert_eq!(sub_args.get_bool("release"), Some(true));
    assert_eq!(sub_args.get_string("target"), Some("wasm"));
}

// ─── Help generation ────────────────────────────────────────────────────────

#[test]
fn help_text_contains_program_info_and_args() {
    let parser = ArgParser::new("mytool")
        .description("A great tool")
        .version("1.2.3")
        .arg(
            Arg::new("output", ArgType::String)
                .long("output")
                .short('o')
                .description("Output file")
                .required(true)
                .default_value("out.txt"),
        )
        .subcommand(Subcommand::new("run", "Run the program"));

    let help = parser.generate_help();
    assert!(help.contains("A great tool"), "missing description");
    assert!(help.contains("mytool 1.2.3"), "missing version line");
    assert!(help.contains("USAGE:"), "missing USAGE section");
    assert!(help.contains("<COMMAND>"), "missing subcommand indicator");
    assert!(help.contains("[OPTIONS]"), "missing options indicator");
    assert!(help.contains("COMMANDS:"), "missing COMMANDS section");
    assert!(help.contains("run"), "missing subcommand name");
    assert!(help.contains("OPTIONS:"), "missing OPTIONS section");
    assert!(help.contains("-o, --output"), "missing flag display");
    assert!(help.contains("[required]"), "missing required marker");
    assert!(
        help.contains("[default: out.txt]"),
        "missing default marker"
    );
    assert!(help.contains("--help"), "missing built-in help flag");
    assert!(help.contains("--version"), "missing built-in version flag");
}

// ─── Subcommand help generation ─────────────────────────────────────────────

#[test]
fn subcommand_help_generation() {
    let parser = ArgParser::new("hud").subcommand(
        Subcommand::new("test", "Run tests").arg(
            Arg::new("filter", ArgType::String)
                .long("filter")
                .description("Filter pattern"),
        ),
    );

    let help = parser.generate_subcommand_help("test").unwrap();
    assert!(help.contains("hud test"));
    assert!(help.contains("Run tests"));
    assert!(help.contains("--filter"));

    // Non-existent subcommand returns None
    assert!(parser.generate_subcommand_help("nonexistent").is_none());
}

// ─── Completion generation ──────────────────────────────────────────────────

#[test]
fn completion_scripts_contain_program_name() {
    let parser = ArgParser::new("hud")
        .arg(
            Arg::new("verbose", ArgType::Bool)
                .long("verbose")
                .short('v'),
        )
        .subcommand(Subcommand::new("build", "Build project"));

    let bash = parser.generate_completion(Shell::Bash);
    assert!(
        bash.contains("_hud_completions"),
        "bash: missing function name"
    );
    assert!(bash.contains("--verbose"), "bash: missing flag");
    assert!(bash.contains("build"), "bash: missing subcommand");

    let zsh = parser.generate_completion(Shell::Zsh);
    assert!(zsh.contains("#compdef hud"), "zsh: missing compdef");
    assert!(zsh.contains("--verbose"), "zsh: missing flag");

    let fish = parser.generate_completion(Shell::Fish);
    assert!(fish.contains("complete -c hud"), "fish: missing complete");
    assert!(fish.contains("build"), "fish: missing subcommand");
}

// ─── --help / --version intercepted as errors ───────────────────────────────

#[test]
fn help_flag_returns_error_with_help_text() {
    let parser = ArgParser::new("app").description("My app").version("0.1.0");
    let err = parser.parse(&args(&["--help"])).unwrap_err();
    match err {
        ArgError::Other(msg) => assert!(msg.contains("My app")),
        other => panic!("expected Other with help text, got {:?}", other),
    }
}

#[test]
fn version_flag_returns_error_with_version() {
    let parser = ArgParser::new("app").version("2.5.0");
    let err = parser.parse(&args(&["--version"])).unwrap_err();
    match err {
        ArgError::Other(msg) => assert!(msg.contains("app 2.5.0")),
        other => panic!("expected Other with version, got {:?}", other),
    }
}

// ─── Arg builder methods ────────────────────────────────────────────────────

#[test]
fn arg_builder_chain() {
    let arg = Arg::new("count", ArgType::Int)
        .short('c')
        .long("count")
        .description("Number of items")
        .required(true)
        .default_value("10");

    assert_eq!(arg.name, "count");
    assert_eq!(arg.short, Some('c'));
    assert_eq!(arg.long.as_deref(), Some("count"));
    assert_eq!(arg.description, "Number of items");
    assert!(arg.required);
    assert_eq!(arg.default_value.as_deref(), Some("10"));
    assert_eq!(arg.arg_type, ArgType::Int);
}

// ─── List default value ─────────────────────────────────────────────────────

#[test]
fn list_default_value_is_split_by_comma() {
    let parser = ArgParser::new("test").arg(
        Arg::new("envs", ArgType::List)
            .long("envs")
            .default_value("dev, staging, prod"),
    );

    let result = parser.parse(&args(&[])).unwrap();
    let list = result.get_list("envs").unwrap();
    assert_eq!(list, &["dev", "staging", "prod"]);
}
