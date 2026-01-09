// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! MIR node definitions
//!
//! MIR is based on basic blocks and explicit control flow.

use crate::ty::MirTy;
use std::collections::HashMap;

/// Unique identifier for MIR nodes and temporaries
pub type MirNodeId = usize;

/// Temporary variable (e.g., _0, _1, _2, ...)
pub type TempVar = usize;

/// MIR function (compilation unit)
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Function name
    pub name: String,

    /// Parameters
    pub params: Vec<MirParam>,

    /// Return type
    pub return_type: MirTy,

    /// Basic blocks in the function
    pub blocks: HashMap<MirNodeId, MirBasicBlock>,

    /// Entry block ID
    pub entry_block: MirNodeId,

    /// Effect handlers (for try...with blocks)
    pub handlers: Vec<MirEffectHandler>,

    /// Effects declared by this function (e.g., ["Log"] for fn() -> i32 | Log)
    pub effects: Vec<String>,

    /// Next available node ID
    pub next_id: MirNodeId,

    /// Next available temporary variable
    pub next_temp: TempVar,
}

impl MirFunction {
    /// Create a new MIR function
    pub fn new(name: String, params: Vec<MirParam>, return_type: MirTy) -> Self {
        let entry_block = 0;
        let mut func = MirFunction {
            name,
            params,
            return_type,
            blocks: HashMap::new(),
            entry_block,
            handlers: Vec::new(),
            effects: Vec::new(),
            next_id: 1,
            next_temp: 0,
        };

        // Create entry block
        func.blocks.insert(entry_block, MirBasicBlock::new(entry_block));
        func
    }

    /// Allocate a new basic block
    pub fn alloc_block(&mut self) -> MirNodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.blocks.insert(id, MirBasicBlock::new(id));
        id
    }

    /// Allocate a new temporary variable
    pub fn alloc_temp(&mut self) -> TempVar {
        let temp = self.next_temp;
        self.next_temp += 1;
        temp
    }
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct MirParam {
    pub name: String,
    pub ty: MirTy,
}

/// Basic block - sequence of instructions without internal control flow
#[derive(Debug, Clone)]
pub struct MirBasicBlock {
    /// Block ID
    pub id: MirNodeId,

    /// Instructions in this block
    pub instructions: Vec<MirInstruction>,

    /// Terminator (control flow at end of block)
    pub terminator: Option<MirTerminator>,
}

impl MirBasicBlock {
    /// Create a new basic block
    pub fn new(id: MirNodeId) -> Self {
        MirBasicBlock {
            id,
            instructions: Vec::new(),
            terminator: None,
        }
    }

    /// Add an instruction to the block
    pub fn push_instruction(&mut self, inst: MirInstruction) {
        self.instructions.push(inst);
    }

    /// Set the terminator (ends the block)
    pub fn set_terminator(&mut self, term: MirTerminator) {
        self.terminator = Some(term);
    }
}

/// MIR instruction (simplified, no nesting)
#[derive(Debug, Clone)]
pub enum MirInstruction {
    /// Assign a constant
    Const {
        dest: TempVar,
        value: MirConstant,
        ty: MirTy,
    },

    /// Copy a variable (if copy type)
    Copy {
        dest: TempVar,
        src: MirPlace,
    },

    /// Move a variable (consume source)
    Move {
        dest: TempVar,
        src: MirPlace,
    },

    /// Binary operation
    BinaryOp {
        dest: TempVar,
        op: MirBinOp,
        left: TempVar,
        right: TempVar,
        ty: MirTy,
    },

    /// Unary operation
    UnaryOp {
        dest: TempVar,
        op: MirUnaryOp,
        operand: TempVar,
        ty: MirTy,
    },

    /// Function call
    Call {
        dest: Option<TempVar>,  // None if function returns unit
        func: MirPlace,
        args: Vec<MirPlace>,
        return_type: MirTy,
    },

    /// Load from a place
    Load {
        dest: TempVar,
        src: MirPlace,
        ty: MirTy,
    },

    /// Store to a place
    Store {
        dest: MirPlace,
        src: TempVar,
        ty: MirTy,
    },

    /// Borrow operation
    Borrow {
        dest: TempVar,
        src: MirPlace,
        mutable: bool,
        ty: MirTy,
    },

    /// Field access (get element pointer)
    FieldAccess {
        dest: TempVar,
        base: TempVar,
        field_name: String,
        field_index: usize,
        ty: MirTy,
    },

    /// Drop a value (run destructor if needed)
    Drop {
        place: MirPlace,
        ty: MirTy,
    },

    /// Perform an effect operation (to be handled by try...with blocks)
    PerformEffect {
        dest: Option<TempVar>,  // None if effect operation returns unit
        effect_name: String,
        operation_name: String,
        args: Vec<MirPlace>,
        return_type: MirTy,
    },
}

/// Constant value
#[derive(Debug, Clone)]
pub enum MirConstant {
    Bool(bool),
    Integer(i128),
    Float(f64),
    Char(char),
    String(String),
    Unit,
}

/// Place in memory (variable, temporary, field, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MirPlace {
    /// Local variable
    Local(String),

    /// Temporary variable
    Temp(TempVar),

    /// Parameter
    Param(String),

    /// Field access: base.field
    Field {
        base: Box<MirPlace>,
        field: String,
    },

    /// Index access: base[index]
    Index {
        base: Box<MirPlace>,
        index: TempVar,
    },

    /// Dereference: *place
    Deref(Box<MirPlace>),

    /// Reference place (for borrow checking)
    Ref {
        place: Box<MirPlace>,
        mutable: bool,
    },
}

/// Binary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    LeftShift, RightShift,
    And, Or,
    Eq, NotEq, Less, LessEq, Greater, GreaterEq,
}

/// Unary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnaryOp {
    Neg, Not, Deref,
    Ref, RefMut,
}

/// Terminator - ends a basic block with control flow
#[derive(Debug, Clone)]
pub enum MirTerminator {
    /// Return from function
    Return(Option<MirPlace>),

    /// Unconditional jump
    Goto {
        target: MirNodeId,
    },

    /// Conditional branch
    If {
        condition: TempVar,
        then_block: MirNodeId,
        else_block: MirNodeId,
    },

    /// Switch (for match expressions)
    Switch {
        scrutinee: TempVar,
        targets: Vec<(MirConstant, MirNodeId)>,
        default: MirNodeId,
    },

    /// Effect operation call (with handler dispatch)
    EffectCall {
        effect_name: String,
        operation_name: String,
        args: Vec<MirPlace>,
        return_type: MirTy,
        /// Where to resume after handler completes (for deep handlers)
        resume_block: MirNodeId,
        /// Destination for return value (None if operation returns unit)
        dest: Option<TempVar>,
    },

    /// Unreachable (for ! type)
    Unreachable,
}

/// Effect handler in MIR
#[derive(Debug, Clone)]
pub struct MirEffectHandler {
    /// Effect name
    pub effect_name: String,

    /// Handler methods (operation implementations)
    /// Maps operation name to (handler_block_id, resume_block_id)
    pub methods: std::collections::HashMap<String, (MirNodeId, MirNodeId)>,
}

/// MIR function (compilation unit)

/// MIR body (collection of functions)
#[derive(Debug, Clone)]
pub struct MirBody {
    pub functions: Vec<MirFunction>,
}

impl MirBody {
    /// Create a new MIR body
    pub fn new() -> Self {
        MirBody {
            functions: Vec::new(),
        }
    }

    /// Add a function to the body
    pub fn push_function(&mut self, func: MirFunction) {
        self.functions.push(func);
    }
}
