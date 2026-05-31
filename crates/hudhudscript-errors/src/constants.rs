// Shared runtime constants — Issue #911.
// These live here (lowest-level shared crate) so every consumer agrees on limits.

/// Maximum call/recursion depth for both interpreter and VM.
pub const MAX_CALL_DEPTH: usize = 2000;
/// Default stack size limit for VM.
pub const MAX_STACK_SIZE: usize = 1_000_000;
/// Actor mailbox capacity.
pub const MAILBOX_CAPACITY: usize = 128;
