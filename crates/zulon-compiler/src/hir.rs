use crate::diag::Span;
use crate::ids::{DefId, EffectId, HirExprId, HirItemId, TyId};
use crate::intern::Symbol;

#[derive(Clone, Debug, Default)]
pub struct HirCrate {
    pub items: Vec<HirItem>,
    pub entry: Option<DefId>,
}

#[derive(Clone, Debug)]
pub enum HirItem {
    Func(HirFunc),
    Type(HirTypeDef),
    Trait(HirTrait),
    Impl(HirImpl),
    Const(HirConst),
}

#[derive(Clone, Debug)]
pub struct HirFunc {
    pub id: HirItemId,
    pub def: DefId,
    pub name: Symbol,
    pub generics: HirGenerics,
    pub params: Vec<HirParam>,
    pub ret: HirTy,
    pub throws: Option<HirErrorTy>,
    pub performs: HirEffectSet,
    pub body: HirBody,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct HirGenerics {
    pub params: Vec<Symbol>,
}

#[derive(Clone, Debug)]
pub struct HirParam {
    pub name: Symbol,
    pub ty: HirTy,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirBody {
    pub exprs: Vec<HirExpr>,
    pub root: HirExprId,
}

#[derive(Clone, Debug)]
pub enum HirStmt {
    Let {
        pat: HirPat,
        init: HirExprId,
        span: Span,
    },
    Expr(HirExprId),
}

#[derive(Clone, Debug)]
pub enum HirExpr {
    Lit(HirLit, Span),
    Var(DefId, Span),

    Let {
        pat: HirPat,
        init: HirExprId,
        span: Span,
    },

    Block {
        stmts: Vec<HirStmt>,
        tail: Option<HirExprId>,
        span: Span,
    },

    Call {
        callee: HirCallee,
        args: Vec<HirExprId>,
        span: Span,
    },

    Match {
        scrut: HirExprId,
        arms: Vec<HirArm>,
        span: Span,
    },

    If {
        cond: HirExprId,
        then_: HirExprId,
        else_: Option<HirExprId>,
        span: Span,
    },

    Coalesce {
        lhs: HirExprId,
        rhs: HirExprId,
        span: Span,
    },

    Try {
        expr: HirExprId,
        span: Span,
    },

    Do {
        op: HirEffectOp,
        args: Vec<HirExprId>,
        span: Span,
    },

    Handle {
        body: HirExprId,
        handlers: Vec<HirHandler>,
        span: Span,
    },

    Intrinsic {
        kind: HirIntrinsic,
        args: Vec<HirExprId>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum HirLit {
    Int(i128),
    Bool(bool),
    Str(Symbol),
}

#[derive(Clone, Debug)]
pub enum HirCallee {
    Def(DefId),
    // Method calls / UFCS can be desugared to Def+explicit self arg in P0.
}

#[derive(Clone, Debug)]
pub struct HirArm {
    pub pat: HirPat,
    pub guard: Option<HirExprId>,
    pub body: HirExprId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum HirPat {
    Wild(Span),
    Bind {
        name: Symbol,
        def: DefId,
        span: Span,
    },
    Variant {
        adt: DefId,
        variant: DefId,
        fields: Vec<HirPat>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct HirEffectSet {
    pub effects: Vec<EffectId>,
}

#[derive(Clone, Debug)]
pub struct HirErrorTy {
    pub alts: Vec<TyId>,
}

#[derive(Clone, Debug)]
pub enum HirTy {
    Named(DefId, Span),
    App {
        ctor: DefId,
        args: Vec<HirTy>,
        span: Span,
    },
    Nullable(Box<HirTy>, Span),
}

#[derive(Clone, Debug)]
pub struct HirEffectOp {
    pub effect: EffectId,
    pub op: Symbol,
}

#[derive(Clone, Debug)]
pub struct HirHandler {
    pub effect: EffectId,
    pub op: Symbol,
    pub params: Vec<(Symbol, HirTy)>,
    pub body: HirExprId,
    pub span: Span,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HirIntrinsic {
    TaskScope,
    TaskSpawn,
    TaskJoin,
    TaskCancel,
    SpawnDetached,
    Region,
    Share,
    Actor,
    Channel,
}

#[derive(Clone, Debug)]
pub struct HirTypeDef {
    pub def: DefId,
    pub name: Symbol,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirTrait {
    pub def: DefId,
    pub name: Symbol,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirImpl {
    pub def: DefId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct HirConst {
    pub def: DefId,
    pub name: Symbol,
    pub span: Span,
}
