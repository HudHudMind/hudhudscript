//! Language-specific keyword mappings
//!
//! All 23 supported languages are now modularized.
//! Each language has its own module with keyword mappings.

// Core languages (originally inline in keyword_map.rs, now modularized)
pub mod ar; // Arabic
pub mod de; // German
pub mod en; // English
pub mod es; // Spanish
pub mod fr; // French
pub mod it; // Italian
pub mod ja; // Japanese
pub mod pt;
pub mod ru; // Russian
pub mod tr; // Turkish
pub mod zh; // Chinese // Portuguese

// Extended languages (already modularized)
pub mod bn;
pub mod bs; // Bosnian
pub mod el; // Greek
pub mod fa; // Persian
pub mod hi; // Hindi
pub mod hr; // Croatian
pub mod id; // Indonesian
pub mod ko; // Korean
pub mod ku; // Kurdish
pub mod pl; // Polish
pub mod sr; // Serbian
pub mod th; // Thai
pub mod vi; // Vietnamese // Bengali
