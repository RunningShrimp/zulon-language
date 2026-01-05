use crate::ids::{CrateId, DefId};

#[derive(Clone, Debug, Default)]
pub struct DefMap {
    pub crate_id: CrateId,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ItemIndex {
    pub items: Vec<DefId>,
}
