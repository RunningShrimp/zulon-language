use crate::ids::{DefId, HirExprId, TyId};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TyKind {
    Unknown,
    Named(DefId),
    Nullable(TyId),
    // `T ! E` 的错误部分建议在签名/throws 表里统一管理；
    // 这里预留扩展空间。
}

#[derive(Default)]
pub struct TyInterner {
    kinds: Vec<TyKind>,
}

impl TyInterner {
    pub fn intern(&mut self, kind: TyKind) -> TyId {
        let id = TyId(self.kinds.len() as u32);
        self.kinds.push(kind);
        id
    }

    pub fn kind(&self, id: TyId) -> Option<&TyKind> {
        self.kinds.get(id.0 as usize)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TyTables {
    pub expr_tys: HashMap<HirExprId, TyId>,
    pub item_tys: HashMap<DefId, TyId>,
}
