//! Governance declaration parsing (constitution, law, council)
//!
//! This module handles all governance-related declarations.
//! Grouped together as they are closely related (cohesion principle).

pub mod constitution;
pub mod council;
pub mod governance_decl;
pub mod law;
pub mod protocol;
pub mod role;

pub use constitution::*;
pub use council::*;
pub use governance_decl::*;
pub use law::*;
pub use protocol::*;
pub use role::*;

use pest::iterators::Pair;

use crate::pest_parser::Rule;

/// Governance model normalization
fn normalize_governance(s: &str) -> String {
    match s {
        "demokrasi" => "democracy",
        "monarşi" => "monarchy",
        "teknokrasi" => "technocracy",
        "teokrasi" => "theocracy",
        "parlamenter" => "parliamentary",
        "liyakat" => "meritocracy",
        "anarşi" => "anarchy",
        "oligarşi" => "oligarchy",
        "uzlaşı" => "consensus",
        "otokrasi" => "autocracy",
        _ => s,
    }
    .to_string()
}

fn normalize_execution(s: &str) -> String {
    match s {
        "paralel" => "parallel",
        "sıralı" => "sequential",
        "rekabetçi" => "competitive",
        "sırayla" => "roundRobin",
        _ => s,
    }
    .to_string()
}
