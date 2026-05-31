use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const PARSE_INVALID_SYNTAX: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(182),
        long_code: "HHS_E_PARSE_INVALID_SYNTAX",
        short_code: "E0182",
        title: "Invalid syntax",
        short_description: "The parser recognized a construct that is grammatically wrong but did not match a more specific rule.",
        long_description: "The parser walked into a state where the input is clearly malformed but does not fall into the more specific buckets like 'unexpected token' or 'unexpected EOF'. The accompanying message describes which rule failed and the location pinpoints where.

Read the message and inspect the line. Common causes are: an expression appearing where a statement is required, a missing operator between two operands, or chaining constructs in an order the grammar does not allow (e.g. `let` inside an expression position).

Reducing the snippet around the highlighted region usually exposes the structural mistake quickly.",
        hints: &["Look at the line above as well — many syntax errors are caused by a missing terminator earlier", "Try removing the offending statement to see if the rest parses cleanly", "Check that braces, parens and brackets are balanced near the location"],
        example_bad: Some("fn main() { let = 5; }"),
        example_good: Some("fn main() { let x = 5; }"),
        see_also: &["HHS_E_PARSE_UNEXPECTED_TOKEN", "HHS_E_PARSE_UNEXPECTED_EOF"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub const PARSE_LEXER_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(183),
        long_code: "HHS_E_PARSE_LEXER_ERROR",
        short_code: "E0183",
        title: "Lexer error surfaced during parsing",
        short_description: "The parser propagated a tokenization error from the lexer.",
        long_description: "While requesting the next token from the lexer, the parser received a lexer error. The wrapped error contains the real cause — typically an invalid character, malformed number, bad escape, or unterminated string.

Fix the underlying lexical problem first. Once tokenization succeeds, the parser can continue. The location reported here usually matches the lexer's own location.

If you see this error and the wrapped lexer message looks fine, double-check for invisible Unicode characters in your source file.",
        hints: &["Open the wrapped lexer error to see the real cause", "Fix lex-level issues before chasing parser ones", "Re-save the file as UTF-8 if the source looks clean to your eye"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LEX_UNEXPECTED_CHAR", "HHS_E_LEX_UNTERMINATED_STRING", "HHS_E_LEX_INVALID_NUMBER"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub const PARSE_UNEXPECTED_EOF: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(184),
        long_code: "HHS_E_PARSE_UNEXPECTED_EOF",
        short_code: "E0184",
        title: "Unexpected end of file while parsing",
        short_description: "The parser ran out of input before finishing the current construct.",
        long_description: "The parser was in the middle of parsing something — a function body, a block, an expression, an argument list — and reached the end of the file before seeing the token that would close it. This almost always means a missing `}`, `)`, `]`, or `;`.

Search backwards from the end of the file for unmatched openers. Most editors have a 'jump to matching bracket' command that makes this fast. Adding the missing terminator usually fixes the error immediately.

If the file looks balanced, check whether a nested construct (a string, a block) was left open earlier — the imbalance can come from far away.",
        hints: &["Look for an unclosed { ( [ or unterminated string above the EOF", "Use your editor's bracket-matching to walk from the last brace upward", "Run a formatter — it often points at the first mis-balanced line"],
        example_bad: Some("fn main() { let x = 5;"),
        example_good: Some("fn main() { let x = 5; }"),
        see_also: &["HHS_E_PARSE_UNEXPECTED_TOKEN", "HHS_E_PARSE_INVALID_SYNTAX"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub const PARSE_UNEXPECTED_TOKEN: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(185),
        long_code: "HHS_E_PARSE_UNEXPECTED_TOKEN",
        short_code: "E0185",
        title: "Unexpected token in input",
        short_description: "The parser expected one token but found a different one.",
        long_description: "At the highlighted position the grammar required a specific token (or one of a small set), but the next token in the stream was something else. The error message lists the expected and the actual token.

Most of the time this is a missing or extra punctuation mark: a forgotten `;`, `,` or `}`, a stray keyword, or a typo in an identifier. Look at the expected list — it tells you exactly what would make the parser happy.

If the expected token seems strange (e.g. `}` when you are clearly in the middle of an expression), the real mistake is usually one or two lines earlier where a previous construct was not properly closed.",
        hints: &["Read the 'expected' list — it tells you what the grammar wants here", "The actual mistake is often on the line before the highlighted one", "Missing semicolons and commas are the most frequent cause"],
        example_bad: Some("let x = 5
let y = 10;"),
        example_good: Some("let x = 5;
let y = 10;"),
        see_also: &["HHS_E_PARSE_UNEXPECTED_EOF", "HHS_E_PARSE_INVALID_SYNTAX"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub const SKILL_PARSE_VALIDATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(261),
        long_code: "HHS_E_SKILL_PARSE_VALIDATION_ERROR",
        short_code: "E0261",
        title: "Skill definition failed validation",
        short_description: "A skill (rule pack) definition was syntactically valid YAML but failed semantic validation.",
        long_description: "The skill loader successfully parsed the YAML for a rule pack, but a validation pass rejected its contents. Typical causes: a required field is missing, a field has the wrong type, a referenced rule does not exist, or a constraint (e.g. unique IDs, non-empty patterns) is violated.

Read the wrapped validation message — it identifies the failing field and reason. Update the skill manifest to satisfy the schema and reload.

If you are authoring a new skill, consult the skill schema documentation; running the skill linter locally before deploying catches most of these problems.",
        hints: &["Check the wrapped message for the exact field that failed", "Run the skill linter locally before pushing changes", "Make sure rule IDs are unique across the pack"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_SKILL_PARSE_YAML_ERROR"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub const SKILL_PARSE_YAML_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(262),
        long_code: "HHS_E_SKILL_PARSE_YAML_ERROR",
        short_code: "E0262",
        title: "Skill definition has malformed YAML",
        short_description: "The YAML parser could not read a skill manifest because of a syntax error.",
        long_description: "The skill loader tried to parse a rule pack's YAML file and failed at the YAML syntax level — before any HudHudScript-specific validation. The wrapped error contains the YAML parser's diagnostic, including the line and column.

Fix the YAML: most often this is a misindented block, an unquoted string containing a special character (`:`, `-`, `#`), or a tab where spaces are required. Pasting the file into a YAML linter will quickly highlight the problem.

Once the YAML parses cleanly, the loader will run the semantic validation pass — see the related validation error if that fails next.",
        hints: &["Use spaces for indentation; YAML rejects tabs at structural positions", "Quote strings containing :, #, or leading dashes", "Run the file through a YAML linter to find the precise syntax issue"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_SKILL_PARSE_VALIDATION_ERROR"],
        since_version: "0.1.0",
        category: ErrorCategory::Parse,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    PARSE_INVALID_SYNTAX,
    PARSE_LEXER_ERROR,
    PARSE_UNEXPECTED_EOF,
    PARSE_UNEXPECTED_TOKEN,
    SKILL_PARSE_VALIDATION_ERROR,
    SKILL_PARSE_YAML_ERROR,
];
