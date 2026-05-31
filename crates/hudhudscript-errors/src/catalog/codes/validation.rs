use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ValidationErrorCode {
    /// E0261 — Skill definition failed validation
    SkillParseValidationError = 261,
    /// E0262 — Skill definition has malformed YAML
    SkillParseYamlError = 262,
    /// E0310 — Custom Validation Rule Failed
    ValidationCustom = 310,
    /// E0311 — Value Has Invalid Format
    ValidationInvalidFormat = 311,
    /// E0312 — Value Has Invalid Length
    ValidationInvalidLength = 312,
    /// E0313 — Required Schema Field Missing
    ValidationMissingRequired = 313,
    /// E0314 — Numeric Value Out Of Range
    ValidationOutOfRange = 314,
    /// E0315 — Value Does Not Match Pattern
    ValidationPatternMismatch = 315,
    /// E0316 — Required Field Missing From Input
    ValidationRequiredFieldMissing = 316,
    /// E0317 — Schema Type Mismatch
    ValidationTypeMismatch = 317,
    /// E0318 — Unknown Schema Type
    ValidationUnknownType = 318,
}
