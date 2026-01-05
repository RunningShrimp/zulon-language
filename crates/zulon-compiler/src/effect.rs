use crate::ids::{EffectId, HirExprId};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct EffectSet {
    pub effects: Vec<EffectId>,
}

#[derive(Clone, Debug, Default)]
pub struct EffectTables {
    pub expr_effects: HashMap<HirExprId, EffectSet>,
}
