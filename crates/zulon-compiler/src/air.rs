use crate::diag::Span;
use crate::ids::{AirBlockId, AirFuncId, AirValueId, DefId, EffectId, TyId};
use crate::intern::Symbol;

#[derive(Clone, Debug, Default)]
pub struct AirModule {
    pub funcs: Vec<AirFunc>,
}

#[derive(Clone, Debug)]
pub struct AirFunc {
    pub id: AirFuncId,
    pub def: DefId,
    pub sig: AirSig,
    pub blocks: Vec<AirBlock>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct AirSig {
    pub params: Vec<AirTy>,
    pub ret: AirTy,
    pub throws: Option<AirErrorTy>,
    pub performs: AirEffectSet,
}

#[derive(Clone, Debug, Default)]
pub struct AirEffectSet {
    pub effects: Vec<EffectId>,
}

#[derive(Clone, Debug, Default)]
pub struct AirErrorTy {
    pub alts: Vec<TyId>,
}

#[derive(Clone, Debug)]
pub struct AirBlock {
    pub id: AirBlockId,
    pub insts: Vec<AirInst>,
}

#[derive(Clone, Debug)]
pub struct AirValue {
    pub id: AirValueId,
    pub ty: AirTy,
}

#[derive(Clone, Debug)]
pub enum AirTy {
    Void,
    Scalar,
    Ref,
    Named(DefId),
}

#[derive(Clone, Debug)]
pub enum AirSpawnKind {
    Scoped,
    Detached,
}

#[derive(Clone, Debug)]
pub enum AirInst {
    Call {
        callee: AirValue,
        args: Vec<AirValue>,
        span: Span,
    },
    Return {
        value: Option<AirValue>,
        span: Span,
    },

    // effects
    Perform {
        effect: EffectId,
        op: Symbol,
        args: Vec<AirValue>,
        span: Span,
    },

    // task/actor
    TaskScopeEnter { span: Span },
    TaskSpawn {
        func: AirValue,
        args: Vec<AirValue>,
        kind: AirSpawnKind,
        span: Span,
    },
    TaskJoin { handle: AirValue, span: Span },
    TaskCancel { handle: AirValue, span: Span },
    ActorSend { actor: AirValue, msg: AirValue, span: Span },

    // region/share
    RegionEnter { span: Span },
    RegionExit { span: Span },
    ShareFreeze { value: AirValue, span: Span },
}
