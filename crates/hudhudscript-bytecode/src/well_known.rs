//! Well-known SymId table — interned once, reused everywhere.
//! Eliminates repeated `interner::intern` calls in hot paths.

use crate::interner;
use crate::SymId;
use std::sync::OnceLock;

pub struct WellKnownSyms {
    pub type_: SymId,
    pub template: SymId,
    pub module: SymId,
    pub instance_id: SymId,
    pub parent: SymId,
    pub length: SymId,
    pub hudhud_env: SymId,
    pub view_name: SymId,
    pub user_path: SymId,
    pub server: SymId,
    pub parts: SymId,
    pub actor_id: SymId,
    pub of_subject: SymId,
    pub id_: SymId,
    pub hudhud_exception: SymId,
}

static WK: OnceLock<WellKnownSyms> = OnceLock::new();

pub fn wk() -> &'static WellKnownSyms {
    WK.get_or_init(|| WellKnownSyms {
        type_: SymId(interner::intern("__type").0),
        template: SymId(interner::intern("__template").0),
        module: SymId(interner::intern("__module").0),
        instance_id: SymId(interner::intern("__instance_id").0),
        parent: SymId(interner::intern("__parent__").0),
        length: SymId(interner::intern("length").0),
        hudhud_env: SymId(interner::intern("__hudhud_env").0),
        view_name: SymId(interner::intern("__view_name").0),
        user_path: SymId(interner::intern("__user_path").0),
        server: SymId(interner::intern("__server").0),
        parts: SymId(interner::intern("__parts").0),
        actor_id: SymId(interner::intern("__actor_id").0),
        of_subject: SymId(interner::intern("__of_subject").0),
        id_: SymId(interner::intern("__id").0),
        hudhud_exception: SymId(interner::intern("__hudhud_exception").0),
    })
}
