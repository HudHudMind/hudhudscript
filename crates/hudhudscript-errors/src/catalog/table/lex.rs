use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const LEX_INVALID_ESCAPE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(118),
        long_code: "HHS_E_LEX_INVALID_ESCAPE",
        short_code: "E0118",
        title: "Invalid escape sequence in string literal",
        short_description: "A backslash in a string literal is followed by a character that is not a recognized escape.",
        long_description: "Inside a string literal the lexer saw a backslash followed by a character it does not recognize as an escape sequence. HudHudScript supports the usual set: \\n, \\r, \\t, \\\\, \\\", \\0, plus \\xNN and \\u{...} for byte and Unicode escapes.

Fix it by either using a supported escape, doubling the backslash if you meant a literal backslash, or removing the backslash entirely. If you are pasting a Windows path, prefer raw separators or escape every backslash.

This error commonly appears when copying regex patterns or file paths into source without adjusting escapes.",
        hints: &["Use \\\\ for a literal backslash inside a double-quoted string", "Supported escapes: \\n \\r \\t \\\\ \\\" \\0 \\xNN \\u{1F600}", "For Windows paths, escape each backslash: \"C:\\\\Users\\\\me\""],
        example_bad: Some("let path = \"C:\\Users\\me\";"),
        example_good: Some("let path = \"C:\\\\Users\\\\me\";"),
        see_also: &["HHS_E_LEX_UNTERMINATED_STRING", "HHS_E_LEX_UNEXPECTED_CHAR"],
        since_version: "0.1.0",
        category: ErrorCategory::Lex,
    };

pub const LEX_INVALID_NUMBER: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(119),
        long_code: "HHS_E_LEX_INVALID_NUMBER",
        short_code: "E0119",
        title: "Malformed numeric literal",
        short_description: "A numeric literal could not be parsed (bad digits, multiple dots, stray separators).",
        long_description: "The lexer started reading a number but the resulting text is not a valid HudHudScript numeric literal. Examples include two decimal points, a hex literal containing non-hex digits, an exponent with no digits, or trailing letters that are not a valid type suffix.

Rewrite the literal in a valid form. HudHudScript accepts decimal integers and floats (with optional `_` digit separators), `0x` hex, `0o` octal and `0b` binary integers, and scientific notation like `1.5e10`.

Watch out for locale habits — use `.` as the decimal separator, never `,`.",
        hints: &["Use '.' for decimals; ',' is not allowed inside numbers", "Hex digits must be 0-9 and a-f only — strip any 'g', 'h', etc.", "Use '_' as a thousands separator: 1_000_000 is valid"],
        example_bad: Some("let pi = 3,14;"),
        example_good: Some("let pi = 3.14;"),
        see_also: &["HHS_E_LEX_UNEXPECTED_CHAR"],
        since_version: "0.1.0",
        category: ErrorCategory::Lex,
    };

pub const LEX_UNEXPECTED_CHAR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(120),
        long_code: "HHS_E_LEX_UNEXPECTED_CHAR",
        short_code: "E0120",
        title: "Unexpected character in source",
        short_description: "The lexer encountered a character that does not start any valid token.",
        long_description: "While tokenizing the source file, the lexer encountered a character that is not part of any keyword, identifier, literal, operator, or punctuation in HudHudScript. This usually means a typo, a stray non-printing character (e.g. a smart quote pasted from a document), or use of a symbol that the language does not support.

Fix it by removing or replacing the offending character. If the file came from another editor, check for invisible Unicode characters (zero-width space, BOM, smart quotes).

String literals must use straight double quotes (\"), not curly quotes. The single-quote character is not used for strings in HudHudScript.",
        hints: &["Check for smart quotes or em-dashes pasted from documents", "Some editors insert zero-width or BOM characters — re-save as plain UTF-8", "Use \" for strings; ' is not a string delimiter in HudHudScript"],
        example_bad: Some("let name = ‘Alice’;"),
        example_good: Some("let name = \"Alice\";"),
        see_also: &["HHS_E_LEX_INVALID_ESCAPE", "HHS_E_LEX_UNTERMINATED_STRING"],
        since_version: "0.1.0",
        category: ErrorCategory::Lex,
    };

pub const LEX_UNTERMINATED_STRING: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(121),
        long_code: "HHS_E_LEX_UNTERMINATED_STRING",
        short_code: "E0121",
        title: "Unterminated string literal",
        short_description: "A string literal was opened but the lexer reached end-of-file without a closing quote.",
        long_description: "The lexer saw an opening `\"` and consumed characters looking for the matching closing quote, but reached the end of the file (or a forbidden newline, depending on the literal kind) first. This means somewhere in your source there is a string that was never closed.

Find the unmatched quote and add the missing `\"`. A common cause is an escaped quote at the end (`\"...\\\"`) that accidentally escapes the terminator. Editors with bracket matching usually highlight the offending opener.

If you need a literal containing a newline, make sure the string type you are using actually allows multi-line content.",
        hints: &["Search for an opening \" without a partner above the reported position", "A trailing \\\" escapes the closing quote — remove the backslash", "For multi-line text, use a raw or multi-line string form if available"],
        example_bad: Some("let greeting = \"hello;"),
        example_good: Some("let greeting = \"hello\";"),
        see_also: &["HHS_E_LEX_INVALID_ESCAPE", "HHS_E_LEX_UNEXPECTED_CHAR"],
        since_version: "0.1.0",
        category: ErrorCategory::Lex,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    LEX_INVALID_ESCAPE,
    LEX_INVALID_NUMBER,
    LEX_UNEXPECTED_CHAR,
    LEX_UNTERMINATED_STRING,
];
