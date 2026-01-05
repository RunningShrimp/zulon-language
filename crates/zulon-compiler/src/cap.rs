use crate::ids::{DefId, HirExprId, TyId};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Local,
    Shared,
}

impl Default for Ownership {
    fn default() -> Self {
        Self::Owned
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConcurrencyCap {
    Send,
    Sync,
    Share,
}

#[derive(Clone, Debug, Default)]
pub struct CapInfo {
    pub ownership: Ownership,
    pub send: bool,
    pub sync: bool,
    pub share: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CapTables {
    pub expr_caps: HashMap<HirExprId, CapInfo>,
    pub ty_caps: HashMap<TyId, CapInfo>,
    pub item_caps: HashMap<DefId, CapInfo>,
}
