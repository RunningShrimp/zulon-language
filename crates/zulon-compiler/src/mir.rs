use crate::diag::Span;
use crate::ids::{DefId, MirBlockId, MirLocalId, TyId};
use crate::intern::Symbol;

#[derive(Clone, Debug)]
pub struct MirBody {
    pub def: DefId,
    pub locals: Vec<MirLocalDecl>,
    pub blocks: Vec<MirBasicBlock>,
    pub start: MirBlockId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MirLocalDecl {
    pub ty: TyId,
    pub mutability: MirMut,
    pub debug_name: Option<Symbol>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MirMut {
    Immutable,
    Mutable,
}

#[derive(Clone, Debug)]
pub struct MirBasicBlock {
    pub stmts: Vec<MirStmt>,
    pub term: MirTerminator,
}

#[derive(Clone, Debug)]
pub enum MirStmt {
    Assign(MirPlace, MirRvalue, Span),
    StorageLive(MirLocalId, Span),
    StorageDead(MirLocalId, Span),
    Drop(MirPlace, Span),
    MarkEffectBoundary(MirEffectMark, Span),
    MarkCapBoundary(MirCapMark, Span),
}

#[derive(Clone, Debug)]
pub enum MirTerminator {
    Goto {
        target: MirBlockId,
        span: Span,
    },
    Return {
        span: Span,
    },
    Switch {
        discr: MirOperand,
        targets: Vec<(i128, MirBlockId)>,
        otherwise: MirBlockId,
        span: Span,
    },
    Call {
        func: MirOperand,
        args: Vec<MirOperand>,
        dest: MirPlace,
        target: MirBlockId,
        unwind: Option<MirBlockId>,
        span: Span,
    },
    Throw {
        err: MirOperand,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum MirPlace {
    Local(MirLocalId),
    Field(Box<MirPlace>, u32),
    Deref(Box<MirPlace>),
}

#[derive(Clone, Debug)]
pub enum MirRvalue {
    Use(MirOperand),
    Ref(MirBorrowKind, MirPlace),
    Aggregate(MirAggKind, Vec<MirOperand>),
    BinOp(MirBinOp, MirOperand, MirOperand),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MirBorrowKind {
    Shared,
    Mutable,
}

#[derive(Clone, Debug)]
pub enum MirAggKind {
    Tuple,
    Adt(DefId),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

#[derive(Clone, Debug)]
pub enum MirOperand {
    Copy(MirPlace),
    Move(MirPlace),
    Const(MirConst),
}

#[derive(Clone, Debug)]
pub enum MirConst {
    Int(i128),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub enum MirEffectMark {
    Perform,
    HandleEnter,
    HandleExit,
}

#[derive(Clone, Debug)]
pub enum MirCapMark {
    TaskSpawn,
    ActorSend,
    ShareFreeze,
    RegionEnter,
    RegionExit,
}
