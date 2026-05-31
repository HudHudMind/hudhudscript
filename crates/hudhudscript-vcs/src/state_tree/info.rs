use crate::branch::BranchId;

/// Branch Info
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub id: BranchId,
    pub name: String,
    pub parent: Option<BranchId>,
    pub version: u64,
    pub change_count: usize,
}
