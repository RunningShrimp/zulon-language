// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Error code registry
//!
//! Error codes provide stable identifiers for common error types, allowing
//! users to search for detailed documentation and solutions.

// Error codes will be used when integrating with type checker
#![allow(dead_code)]

/// An error code with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    /// The error code (e.g., "E0308")
    pub code: &'static str,
    /// Category of error (type, name, syntax, etc.)
    pub category: ErrorCategory,
    /// Brief description
    pub description: &'static str,
}

/// Error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Type-related errors
    Type,
    /// Name resolution errors
    Name,
    /// Syntax errors
    Syntax,
    /// Lifetime errors
    Lifetime,
    /// Mutability errors
    Mutability,
    /// Effect system errors
    Effect,
    /// Generic errors
    Generic,
}

// ==================== Type Errors ====================

/// Type mismatch error
pub const E_TYPE_MISMATCH: ErrorCode = ErrorCode {
    code: "E0308",
    category: ErrorCategory::Type,
    description: "type mismatch in expression or function call",
};

/// Type not found error
pub const E_UNDEFINED_TYPE: ErrorCode = ErrorCode {
    code: "E0412",
    category: ErrorCategory::Type,
    description: "type name not found in this scope",
};

/// Cannot call non-function type
pub const E_NOT_CALLABLE: ErrorCode = ErrorCode {
    code: "E0618",
    category: ErrorCategory::Type,
    description: "attempted to call a non-function type",
};

/// Arity mismatch (wrong number of arguments)
pub const E_ARITY_MISMATCH: ErrorCode = ErrorCode {
    code: "E0061",
    category: ErrorCategory::Type,
    description: "function call with wrong number of arguments",
};

/// Field does not exist on type
pub const E_UNKNOWN_FIELD: ErrorCode = ErrorCode {
    code: "E0609",
    category: ErrorCategory::Type,
    description: "no such field on type",
};

/// Type is not indexable
pub const E_NOT_INDEXABLE: ErrorCode = ErrorCode {
    code: "E0608",
    category: ErrorCategory::Type,
    description: "type cannot be indexed",
};

/// Integer literal overflow
pub const E_INT_OVERFLOW: ErrorCode = ErrorCode {
    code: "E0080",
    category: ErrorCategory::Type,
    description: "integer literal too large to fit in target type",
};

/// Cannot convert between types
pub const E_CANNOT_CONVERT: ErrorCode = ErrorCode {
    code: "E0604",
    category: ErrorCategory::Type,
    description: "cannot convert between these types",
};

/// Recursive type definition
pub const E_RECURSIVE_TYPE: ErrorCode = ErrorCode {
    code: "E0072",
    category: ErrorCategory::Type,
    description: "recursive type contains itself",
};

// ==================== Name Errors ====================

/// Undefined variable
pub const E_UNDEFINED_VARIABLE: ErrorCode = ErrorCode {
    code: "E0425",
    category: ErrorCategory::Name,
    description: "cannot find value in this scope",
};

/// Undefined function
pub const E_UNDEFINED_FUNCTION: ErrorCode = ErrorCode {
    code: "E0422",
    category: ErrorCategory::Name,
    description: "cannot find function in this scope",
};

/// Undefined effect
pub const E_UNDEFINED_EFFECT: ErrorCode = ErrorCode {
    code: "E0000",
    category: ErrorCategory::Name,
    description: "cannot find effect in this scope",
};

/// Missing generic parameter
pub const E_MISSING_GENERIC: ErrorCode = ErrorCode {
    code: "E0392",
    category: ErrorCategory::Name,
    description: "generic parameter not provided",
};

// ==================== Mutability Errors ====================

/// Cannot assign to immutable value
pub const E_CANNOT_ASSIGN_IMMUTABLE: ErrorCode = ErrorCode {
    code: "E0384",
    category: ErrorCategory::Mutability,
    description: "cannot assign to immutable variable",
};

/// Cannot borrow as mutable
pub const E_CANNOT_BORROW_MUT: ErrorCode = ErrorCode {
    code: "E0596",
    category: ErrorCategory::Mutability,
    description: "cannot borrow as mutable",
};

// ==================== Trait/Generic Errors ====================

/// Trait bound not satisfied
pub const E_TRAIT_NOT_SATISFIED: ErrorCode = ErrorCode {
    code: "E0277",
    category: ErrorCategory::Generic,
    description: "trait bound not satisfied",
};

/// Type inference error
pub const E_INFERENCE_ERROR: ErrorCode = ErrorCode {
    code: "E0282",
    category: ErrorCategory::Generic,
    description: "type inference failed",
};

impl ErrorCode {
    /// Get the error code string
    pub fn as_str(&self) -> &'static str {
        self.code
    }

    /// Get the category
    pub fn category(&self) -> ErrorCategory {
        self.category
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        self.description
    }

    /// Format error code with description
    pub fn format(&self) -> String {
        format!("{}: {}", self.code, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        let code = E_TYPE_MISMATCH;
        assert_eq!(code.as_str(), "E0308");
        assert_eq!(code.category(), ErrorCategory::Type);
        assert!(!code.description().is_empty());
    }

    #[test]
    fn test_error_codes_are_unique() {
        let codes = vec![
            E_TYPE_MISMATCH,
            E_UNDEFINED_TYPE,
            E_NOT_CALLABLE,
            E_ARITY_MISMATCH,
            E_UNKNOWN_FIELD,
            E_NOT_INDEXABLE,
            E_INT_OVERFLOW,
            E_CANNOT_CONVERT,
            E_RECURSIVE_TYPE,
            E_UNDEFINED_VARIABLE,
            E_UNDEFINED_FUNCTION,
            E_UNDEFINED_EFFECT,
            E_MISSING_GENERIC,
            E_CANNOT_ASSIGN_IMMUTABLE,
            E_CANNOT_BORROW_MUT,
            E_TRAIT_NOT_SATISFIED,
            E_INFERENCE_ERROR,
        ];

        let unique_codes: std::collections::HashSet<&'static str> =
            codes.iter().map(|c| c.code).collect();

        assert_eq!(unique_codes.len(), codes.len(), "Error codes must be unique");
    }
}
