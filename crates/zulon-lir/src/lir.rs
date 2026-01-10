// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LIR node definitions
//!
//! LIR is based on SSA (Static Single Assignment) form.

use crate::ty::LirTy;
use std::collections::HashMap;

/// Unique identifier for LIR nodes
pub type LirNodeId = usize;

/// Virtual register (SSA value)
pub type VReg = u32;

/// LIR function (compilation unit in SSA form)
#[derive(Debug, Clone)]
pub struct LirFunction {
    /// Function name
    pub name: String,

    /// Parameters (as virtual registers)
    pub params: Vec<VReg>,

    /// Parameter types
    pub param_types: Vec<LirTy>,

    /// Return type
    pub return_type: LirTy,

    /// Basic blocks in the function
    pub blocks: HashMap<LirNodeId, LirBlock>,

    /// Entry block ID
    pub entry_block: LirNodeId,

    /// Next available node ID
    pub next_id: LirNodeId,

    /// Next available virtual register
    pub next_vreg: VReg,

    /// External function names (for calls)
    pub external_funcs: Vec<String>,
}

impl LirFunction {
    /// Create a new LIR function
    pub fn new(
        name: String,
        params: Vec<(VReg, LirTy)>,
        return_type: LirTy,
    ) -> Self {
        let entry_block = 0;
        let (param_regs, param_types): (Vec<_>, Vec<_>) = params.into_iter().unzip();

        let mut func = LirFunction {
            name,
            params: param_regs.clone(),
            param_types,
            return_type,
            blocks: HashMap::new(),
            entry_block,
            next_id: 1,
            next_vreg: param_regs.iter().copied().max().map_or(0, |m| m + 1),
            external_funcs: Vec::new(),
        };

        // Create entry block with parameter phi nodes
        let mut entry = LirBlock::new(entry_block);
        for (i, &reg) in param_regs.iter().enumerate() {
            // Add phi node for each parameter
            entry.phi_nodes.insert(reg, LirPhi {
                def: reg,
                sources: vec![(reg, entry_block)],
                ty: func.param_types[i].clone(),
            });
        }
        func.blocks.insert(entry_block, entry);

        func
    }

    /// Allocate a new virtual register
    pub fn alloc_vreg(&mut self) -> VReg {
        let reg = self.next_vreg;
        self.next_vreg += 1;
        reg
    }

    /// Allocate a new basic block
    pub fn alloc_block(&mut self) -> LirNodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.blocks.insert(id, LirBlock::new(id));
        id
    }
}

/// Basic block in SSA form
#[derive(Debug, Clone)]
pub struct LirBlock {
    /// Block ID
    pub id: LirNodeId,

    /// Phi nodes (SSA merge points)
    pub phi_nodes: HashMap<VReg, LirPhi>,

    /// Instructions in this block
    pub instructions: Vec<LirInstruction>,

    /// Terminator (control flow at end of block)
    pub terminator: Option<LirTerminator>,
}

impl LirBlock {
    /// Create a new basic block
    pub fn new(id: LirNodeId) -> Self {
        LirBlock {
            id,
            phi_nodes: HashMap::new(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    /// Add an instruction to the block
    pub fn push_instruction(&mut self, inst: LirInstruction) {
        self.instructions.push(inst);
    }

    /// Add a phi node to the block
    pub fn add_phi(&mut self, vreg: VReg, phi: LirPhi) {
        self.phi_nodes.insert(vreg, phi);
    }

    /// Set the terminator (ends the block)
    pub fn set_terminator(&mut self, term: LirTerminator) {
        self.terminator = Some(term);
    }
}

/// Phi node (SSA merge)
#[derive(Debug, Clone)]
pub struct LirPhi {
    /// The virtual register being defined
    pub def: VReg,

    /// Sources: (vreg, predecessor_block)
    pub sources: Vec<(VReg, LirNodeId)>,

    /// Type of the phi node
    pub ty: LirTy,
}

/// LIR instruction (SSA form)
#[derive(Debug, Clone)]
pub enum LirInstruction {
    /// Stack allocation (for mutable variables)
    Alloca(LirAlloca),

    /// Constant
    Const {
        dest: VReg,
        value: LirConstant,
        ty: LirTy,
    },

    /// Copy (register-to-register)
    Copy {
        dest: VReg,
        src: VReg,
        ty: LirTy,
    },

    /// Binary operation
    BinaryOp {
        dest: VReg,
        op: LirBinOp,
        left: VReg,
        right: VReg,
        ty: LirTy,
    },

    /// Unary operation
    UnaryOp {
        dest: VReg,
        op: LirUnaryOp,
        operand: VReg,
        ty: LirTy,
    },

    /// Load from memory
    Load {
        dest: VReg,
        src: LirOperand,
        ty: LirTy,
    },

    /// Store to memory
    Store {
        dest: LirOperand,
        src: VReg,
        ty: LirTy,
    },

    /// Get element pointer (for field/array access)
    Gep {
        dest: VReg,
        base: VReg,
        indices: Vec<LirOperand>,
        ty: LirTy,
    },

    /// Function call
    Call {
        dest: Option<VReg>,
        func: VReg,
        args: Vec<VReg>,
        return_type: LirTy,
    },

    /// External function call (by name)
    CallExternal {
        dest: Option<VReg>,
        func_name: String,
        args: Vec<VReg>,
        arg_types: Vec<LirTy>,
        return_type: LirTy,
    },

    /// Comparison
    Cmp {
        dest: VReg,
        op: LirCmpOp,
        left: VReg,
        right: VReg,
    },

    /// Cast
    Cast {
        dest: VReg,
        src: VReg,
        from: LirTy,
        to: LirTy,
    },

    /// Increment reference count (for Arc<T>)
    RefInc {
        ptr: VReg,
        ty: LirTy,
    },

    /// Decrement reference count (for Arc<T>)
    RefDec {
        ptr: VReg,
        ty: LirTy,
    },
}

/// Constant value
#[derive(Debug, Clone)]
pub enum LirConstant {
    Bool(bool),
    Integer(u64),
    Float(f64),
    String(String),
    Unit,
}

/// Operand (can be register or immediate)
#[derive(Debug, Clone)]
pub enum LirOperand {
    Reg(VReg),
    Imm(u64),
    ImmFloat(f64),
}

/// Binary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirBinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    LeftShift, RightShift,
}

/// Stack slot allocation (for mutable variables)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirAlloca {
    pub dest: VReg,
    pub ty: LirTy,
}

/// Unary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirUnaryOp {
    Neg, Not,
}

/// Comparison operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirCmpOp {
    Eq, NotEq,
    Less, LessEq,
    Greater, GreaterEq,
}

/// Terminator - ends a basic block with control flow
#[derive(Debug, Clone)]
pub enum LirTerminator {
    /// Return from function (normal return)
    Return(Option<VReg>),

    /// Throw an error (error return from functions with error types)
    Throw(VReg),

    /// Unconditional jump
    Jump {
        target: LirNodeId,
    },

    /// Conditional branch
    Branch {
        condition: VReg,
        then_block: LirNodeId,
        else_block: LirNodeId,
    },

    /// Switch (for match expressions)
    Switch {
        scrutinee: VReg,
        targets: Vec<(u64, LirNodeId)>,
        default: LirNodeId,
    },

    /// Unreachable (for ! type)
    Unreachable,
}

/// External function declaration
#[derive(Debug, Clone)]
pub struct LirExternal {
    /// External function name
    pub name: String,

    /// Parameter types
    pub param_types: Vec<LirTy>,

    /// Return type
    pub return_type: LirTy,

    /// Whether this is a variadic function (like printf)
    pub variadic: bool,
}

/// LIR body (collection of functions and external declarations)
#[derive(Debug, Clone)]
pub struct LirBody {
    pub functions: Vec<LirFunction>,
    pub externals: Vec<LirExternal>,
}

impl LirBody {
    /// Create a new LIR body
    pub fn new() -> Self {
        LirBody {
            functions: Vec::new(),
            externals: Vec::new(),
        }
    }

    /// Add a function to the body
    pub fn push_function(&mut self, func: LirFunction) {
        self.functions.push(func);
    }

    /// Add an external function declaration
    pub fn push_external(&mut self, external: LirExternal) {
        self.externals.push(external);
    }
}
