# ZULON Error Handling System Design

**Version**: 1.0
**Date**: 2026-01-08
**Status**: Design Phase
**Priority**: P0 (Critical)

---

## Table of Contents

1. [Overview](#overview)
2. [Syntax Design](#syntax-design)
3. [Type System Integration](#type-system-integration)
4. [Compiler Implementation](#compiler-implementation)
5. [Code Generation Strategy](#code-generation-strategy)
6. [Examples](#examples)
7. [Testing Strategy](#testing-strategy)
8. [Implementation Timeline](#implementation-timeline)

---

## Overview

### Goals

The ZULON error handling system aims to provide:

1. **Explicit Error Handling**: Errors must be acknowledged, not silently ignored
2. **Ergonomic Syntax**: Concise and readable error propagation
3. **Type Safety**: Compile-time guarantees about error handling
4. **Performance**: Zero-cost abstractions where possible
5. **Interoperability**: Work seamlessly with existing Outcome<T, E> type

### Design Principles

1. **Explicit > Implicit**: Errors must be visible in the type signature
2. **Composability**: Easy to combine multiple error-prone operations
3. **Clarity**: Error flow should be obvious to readers
4. **No Exceptions**: No hidden control flow via exceptions

---

## Syntax Design

### 1. Throw Statement

**Purpose**: Explicitly return an error from a function

**Syntax**:
```zulon
throw <expression>;
```

**Examples**:
```zulon
// Basic throw
fn parse_age(s: string) -> Outcome<i32, ParseError> {
    if s.is_empty() {
        throw ParseError::EmptyString;
    }
    // ...
}

// Throw with value
fn divide(a: i32, b: i32) -> Outcome<i32, DivideError> {
    if b == 0 {
        throw DivideError::DivisionByZero { dividend: a };
    }
    // ...
}

// Conditional throw
fn get_user(id: i32) -> Outcome<User, DatabaseError> {
    if id < 0 {
        throw DatabaseError::InvalidId(id);
    }
    // ...
}
```

**Grammar**:
```
<throw_statement> ::= "throw" <expression> ";"
```

### 2. Question Mark Operator (?)

**Purpose**: Early return on error, propagating the error to the caller

**Syntax**:
```zulon<expression> "?"
```

**Examples**:
```zulon
fn read_file(path: string) -> Outcome<string, IoError> {
    let file = File::open(path)?;  // Returns IoError if open fails
    let contents = file.read_to_string()?;  // Returns IoError if read fails
    Outcome::Ok(contents)
}

fn fetch_user(id: i32) -> Outcome<User, Error> {
    let conn = connect_to_database()?;  // Returns Error if connection fails
    let user = conn.query_user(id)?;  // Returns Error if query fails
    Outcome::Ok(user)
}
```

**Desugaring**:
```zulon
// Before (with ?)
let value = risky_operation()?;

// After (desugared)
let value = match risky_operation() {
    Outcome::Ok(v) => v,
    Outcome::Err(e) => throw e,
};
```

**Grammar**:
```
<prefix_expression> ::= <prefix_expression> "?"
```

### 3. Pipe Separator (|) for Multiple Return Values

**Purpose**: Separate success value, error value, and effects in function signatures

**Syntax**:
```zulon
fn <name>(<params>) -> <success_type> | <error_type> | <effects>
```

**Examples**:
```zulon
// Success and error only
fn parse_int(s: string) -> i32 | ParseError {
    // ...
}

// Success, error, and effect
fn write_log(msg: string) -> unit | IoError | Io {
    // ...
}

// Multiple effects
fn handle_request(req: Request) -> Response | HttpError | Io + Database {
    // ...
}

// Generic function with error
fn find_user<K>(key: K) -> Optional<User> | NotFoundError
where K: Hash + Eq
{
    // ...
}
```

**Simplified Forms**:
```zulon
// No error (infallible)
fn always_succeeds() -> i32 {
    // ...
}

// No explicit error (unit error)
fn might_fail() -> i32 || {
    // Equivalent to: i32 | unit
}

// Only error
fn only_errors() -> || IoError {
    // Equivalent to: unit | IoError
}
```

**Grammar**:
```
<return_type> ::=
    | <type>                         // Success only
    | <type> "|" <type>              // Success | Error
    | <type> "|" <type> "|" <effects>  // Success | Error | Effects
    | <type> "||" <type>             // Shorthand for Success | unit | Error
    | "||" <type>                    // Shorthand for unit | Error

<effects> ::= <type> ( "+" <type> )*
```

### 4. Error Type Definition

**Purpose**: Define custom error types

**Syntax**:
```zulon
enum <Name> {
    <Variant1>,
    <Variant2> { <fields> },
    <Variant3>(<tuple_fields>),
}
```

**Examples**:
```zulon
// Simple enum
enum ParseError {
    EmptyString,
    InvalidDigit(char),
    Overflow,
}

// Struct variants
enum DatabaseError {
    ConnectionFailed { message: string },
    InvalidQuery { sql: string, line: i32 },
    Timeout { duration: i32 },
}

// Mixed variants
enum IoError {
    NotFound(string),
    PermissionDenied(string),
    Other { code: i32, message: string },
}

// Generic error
enum Error<T> {
    NotFound,
    Invalid(T),
    Unknown,
}
```

### 5. Error Trait

**Purpose**: Common interface for all error types

**Definition**:
```zulon
trait Error {
    fn description(&self) -> string;
    fn cause(&self) -> Optional<&dyn Error>;
}
```

**Auto-Derivation**:
```zulon
#[derive(Error)]
enum MyError {
    // Automatically implements Error trait
    Variant1,
    Variant2 { field: i32 },
}
```

---

## Type System Integration

### 1. Outcome Type Enhancement

The existing `Outcome<T, E>` type will be enhanced:

```zulon
enum Outcome<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Outcome<T, E> {
    // Constructor methods
    fn ok(value: T) -> Self { Outcome::Ok(value) }
    fn err(error: E) -> Self { Outcome::Err(error) }

    // Query methods
    fn is_ok(&self) -> bool { /* ... */ }
    fn is_err(&self) -> bool { /* ... */ }

    // Accessor methods
    fn unwrap(self) -> T { /* ... */ }
    fn unwrap_err(self) -> E { /* ... */ }
    fn expect(self, msg: string) -> T { /* ... */ }

    // Transformation methods
    fn map<U, F>(self, f: F) -> Outcome<U, E>
    where F: FnOnce(T) -> U { /* ... */ }

    fn map_err<F, G>(self, f: G) -> Outcome<T, F>
    where G: FnOnce(E) -> F { /* ... */ }

    // Combinators
    fn and<U>(self, other: Outcome<U, E>) -> Outcome<U, E> { /* ... */ }
    fn or<F>(self, other: Outcome<T, F>) -> Outcome<T, F> { /* ... */ }
}
```

### 2. Type Inference Rules

#### Rule 1: Throw Statement Type
```
Γ ⊢ throw e : τ
where Γ ⊢ e : E and τ = E

// The type of "throw e" is the error type E
// This can coerce to any T | E where T is the expected success type
```

#### Rule 2: Question Mark Operator
```
Γ ⊢ e? : T | E
where Γ ⊢ e : Outcome<T, E>

// If e has type Outcome<T, E>, then e? has type T | E
// The ? operator "unwraps" the Outcome
```

#### Rule 3: Function Signature with |
```
fn f(...) -> T | E { ... }

// Desugars to:
fn f(...) -> Outcome<T, E> { ... }

// The body is automatically wrapped if needed:
// - If return type is T, wrap in Outcome::Ok
// - If throw is used, wrap in Outcome::Err
```

#### Rule 4: Error Type Coercion
```
T | E <: T | F
where E <: F

// If error type E is a subtype of F, then T | E is a subtype of T | F
// Allows returning specific errors in functions expecting general errors
```

### 3. Generic Error Handling

```zulon
// Generic over error type
fn parse_or_default<T>(s: string, default: T) -> T | ParseError
where T: FromStr
{
    match s.parse() {
        Ok(v) => Ok(v),
        Err(e) => throw e,
    }
}

// Generic over success type, fixed error
fn result_unit<T>() -> T | IoError {
    throw IoError::NotFound("file".to_string());
}
```

---

## Compiler Implementation

### Phase 1: Lexer Changes (Day 1-2)

**File**: `crates/zulon-parser/src/lexer/token.rs`

**Additions**:
```rust
pub enum TokenKind {
    // Existing tokens...
    Throw,          // "throw"
    Question,       // "?"
    Pipe,           // "|"
    DoublePipe,     // "||"

    // ...
}

impl TokenKind {
    pub fn is_throw_keyword(&self) -> bool {
        matches!(self, TokenKind::Throw)
    }
}
```

**File**: `crates/zulon-parser/src/lexer/mod.rs`

**Modifications**:
```rust
impl<'a> Lexer<'a> {
    fn read_identifier(&mut self) -> TokenKind {
        // ... existing code ...

        match ident.as_str() {
            // ... existing keywords ...
            "throw" => TokenKind::Throw,
            _ => TokenKind::Identifier(ident),
        }
    }

    fn read_operator(&mut self) -> TokenKind {
        match self.peek_char() {
            // ... existing operators ...
            '?' => {
                self.advance();
                TokenKind::Question
            }
            '|' => {
                self.advance();
                if self.peek_char() == Some('|') {
                    self.advance();
                    TokenKind::DoublePipe
                } else {
                    TokenKind::Pipe
                }
            }
            // ...
        }
    }
}
```

**Tests**: `crates/zulon-parser/src/lexer/tests.rs`
```rust
#[test]
fn test_throw_keyword() {
    let source = r#"throw error;"#;
    let lexer = Lexer::new(source);
    let (tokens, _) = lexer.lex_all();
    assert_eq!(tokens[0].kind, TokenKind::Throw);
}

#[test]
fn test_question_mark() {
    let source = r#"value?"#;
    let lexer = Lexer::new(source);
    let (tokens, _) = lexer.lex_all();
    assert_eq!(tokens[1].kind, TokenKind::Question);
}

#[test]
fn test_pipe_separator() {
    let source = r#"fn f() -> i32 | Error"#;
    let lexer = Lexer::new(source);
    let (tokens, _) = lexer.lex_all();
    assert_eq!(tokens[8].kind, TokenKind::Pipe);
}
```

### Phase 2: Parser Changes (Day 3-5)

**File**: `crates/zulon-parser/src/ast/mod.rs`

**Additions**:
```rust
/// Throw statement: throw <expression>;
#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStatement {
    pub error: Box<Expression>,
    pub span: Span,
}

/// Function return type with | separator
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionReturnType {
    Simple(Type),
    WithError {
        success: Type,
        error: Type,
        effects: Vec<Type>,  // Optional
    },
    ShorthandError {
        success: Type,
        error: Type,
    },
}
```

**File**: `crates/zulon-parser/src/parser/mod.rs`

**Modifications**:
```rust
impl<'a> Parser<'a> {
    // Parse throw statement
    fn parse_throw_statement(&mut self) -> Result<Statement, ParseError> {
        let throw_span = self.expect_token(TokenKind::Throw)?;

        let error_expr = self.parse_expression()?;

        let semicolon_span = self.expect_token(TokenKind::Semicolon)?;

        Ok(Statement::Throw(ThrowStatement {
            error: Box::new(error_expr),
            span: throw_span.merge(&semicolon_span),
        }))
    }

    // Parse return type with | separator
    fn parse_return_type(&mut self) -> Result<FunctionReturnType, ParseError> {
        let success_type = self.parse_type()?;

        if self.peek_token_kind() == TokenKind::Pipe {
            self.advance();  // consume |

            // Check for || (shorthand)
            if self.peek_token_kind() == TokenKind::Pipe {
                self.advance();  // consume second |
                let error_type = self.parse_type()?;
                return Ok(FunctionReturnType::ShorthandError {
                    success: success_type,
                    error: error_type,
                });
            }

            let error_type = self.parse_type()?;

            // Check for effects (| after error type)
            let mut effects = Vec::new();
            if self.peek_token_kind() == TokenKind::Pipe {
                self.advance();  // consume |

                // Parse effect list
                while self.peek_token_kind() != TokenKind::RightBrace {
                    effects.push(self.parse_type()?);

                    if self.peek_token_kind() == TokenKind::Plus {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }

            Ok(FunctionReturnType::WithError {
                success: success_type,
                error: error_type,
                effects,
            })
        } else {
            Ok(FunctionReturnType::Simple(success_type))
        }
    }

    // Parse ? operator (highest precedence in prefix)
    fn parse_prefix_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        // Check for ?
        if self.peek_token_kind() == TokenKind::Question {
            let question_span = self.advance();
            expr = Expression::QuestionMark {
                expr: Box::new(expr),
                span: expr.span().merge(&question_span),
            };
        }

        Ok(expr)
    }
}
```

**Tests**: `crates/zulon-parser/src/parser/tests.rs`
```rust
#[test]
fn test_throw_statement() {
    let source = r#"
        fn f() -> i32 | Error {
            throw Error::Invalid;
        }
    "#;
    let parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    // Validate AST structure
}

#[test]
fn test_question_mark_operator() {
    let source = r#"
        fn f() -> i32 | Error {
            let x = risky()?;
            Outcome::Ok(x)
        }
    "#;
    let parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    // Validate AST structure
}

#[test]
fn test_pipe_separator() {
    let source = r#"
        fn f1() -> i32 | Error { }
        fn f2() -> i32 || Error { }
        fn f3() -> i32 | Error | Io { }
    "#;
    let parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    // Validate AST structure
}
```

### Phase 3: Type System Integration (Day 6-8)

**File**: `crates/zulon-typeck/src/checker.rs`

**Additions**:
```rust
impl<'tcx> TypeChecker<'tcx> {
    // Type check throw statement
    fn check_throw_statement(
        &mut self,
        stmt: &ThrowStatement,
        expected_return: &FunctionReturnType,
    ) -> Result<(), TypeError> {
        // Type check the error expression
        let error_type = self.check_expression(&stmt.error)?;

        match expected_return {
            FunctionReturnType::Simple(ty) => {
                // No error expected - this is a type error
                Err(TypeError::ThrowInInfallibleFunction {
                    span: stmt.span,
                    function_type: ty.clone(),
                })
            }
            FunctionReturnType::WithError { error, .. } |
            FunctionReturnType::ShorthandError { error, .. } => {
                // Check if error type matches
                if self.types_compatible(&error_type, error) {
                    Ok(())
                } else {
                    Err(TypeError::MismatchedErrorType {
                        span: stmt.span,
                        expected: error.clone(),
                        found: error_type,
                    })
                }
            }
        }
    }

    // Type check ? operator
    fn check_question_mark(
        &mut self,
        expr: &Expression,
        expected: &Type,
    ) -> Result<Type, TypeError> {
        // Get the type of the inner expression
        let inner_type = self.check_expression(expr)?;

        // Inner expression must be Outcome<T, E>
        match &inner_type {
            Type::Outcome { success, error } => {
                // The ? operator returns the success type T
                Ok((*success).clone())
            }
            _ => Err(TypeError::QuestionMarkOnNonOutcome {
                span: expr.span(),
                found: inner_type,
            }),
        }
    }

    // Check function return type
    fn check_function_return_type(
        &mut self,
        decl: &FunctionDecl,
    ) -> Result<FunctionType, TypeError> {
        match &decl.return_type {
            FunctionReturnType::Simple(ty) => {
                Ok(FunctionType {
                    success: ty.clone(),
                    error: None,
                    effects: vec![],
                })
            }
            FunctionReturnType::WithError { success, error, effects } => {
                Ok(FunctionType {
                    success: success.clone(),
                    error: Some(error.clone()),
                    effects: effects.clone(),
                })
            }
            FunctionReturnType::ShorthandError { success, error } => {
                Ok(FunctionType {
                    success: success.clone(),
                    error: Some(error.clone()),
                    effects: vec![],
                })
            }
        }
    }
}
```

---

## Code Generation Strategy

### Phase 4: LLVM IR Generation (Day 9-10)

**File**: `crates/zulon-codegen-llvm/src/lib.rs`

**Strategy**:

#### 1. Throw Statement → Early Return

```zulon
// Source
fn divide(a: i32, b: i32) -> i32 | DivideError {
    if b == 0 {
        throw DivideError::DivisionByZero;
    }
    Outcome::Ok(a / b)
}

// Generated LLVM IR (pseudocode)
define void @divide(i32 %a, i32 %b, {i32, i32}* %result) {
start:
  ; Check if b == 0
  %cmp = icmp eq i32 %b, 0
  br i1 %cmp, label %throw_error, label %continue

throw_error:
  ; Construct error variant
  %error = alloca i32
  store i32 0, i32* %error  ; DivisionByZero discriminant

  ; Store error in result
  %result_err = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 1
  store i32 0, i32* %result_err

  ; Set error flag
  %result_flag = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 0
  store i32 1, i32* %result_flag  ; 1 = error

  ret void

continue:
  ; Normal division
  %quotient = sdiv i32 %a, %b

  ; Store success in result
  %result_val = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 1
  store i32 %quotient, i32* %result_val

  ; Set success flag
  %result_flag = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 0
  store i32 0, i32* %result_flag  ; 0 = success

  ret void
}
```

#### 2. Question Mark → Branch on Error

```zulon
// Source
fn read_and_parse() -> i32 | IoError {
    let s = read_file()?;  // Returns Outcome<string, IoError>
    parse_int(s)
}

// Generated LLVM IR (pseudocode)
define void @read_and_parse({i32, i32}* %result) {
entry:
  ; Call read_file
  %temp1 = alloca {i8*, i32}
  call void @read_file({i8*, i32}* %temp1)

  ; Check if error
  %flag1 = getelementptr {i8*, i32}, {i8*, i32}* %temp1, i32 0, i32 0
  %is_error = load i32, i32* %flag1
  icmp ne i32 %is_error, 0
  br i1 %is_error, label %propagate_error, label %continue

propagate_error:
  ; Copy error to result
  %err_val = getelementptr {i8*, i32}, {i8*, i32}* %temp1, i32 0, i32 1
  %result_err = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 1
  %err = load i32, i32* %err_val
  store i32 %err, i32* %result_err

  ; Set error flag
  %result_flag = getelementptr {i32, i32}, {i32, i32}* %result, i32 0, i32 0
  store i32 1, i32* %result_flag

  ret void

continue:
  ; Extract success value
  %str_val = getelementptr {i8*, i32}, {i8*, i32}* %temp1, i32 0, i32 1
  %s = load i8*, i8** %str_val

  ; Call parse_int with s
  %temp2 = alloca {i32, i32}
  call void @parse_int(i8* %s, {i32, i32}* %temp2)

  ; Copy result
  ; ... (similar pattern)

  ret void
}
```

#### 3. Outcome Representation

```rust
// Outcome<T, E> is represented as a tagged union:
// struct Outcome<T, E> {
//     is_error: bool,  // false = Ok, true = Err
//     value: [T; E],   // Either T or E (overlapping storage)
// }
```

---

## Examples

### Example 1: Basic Error Handling

```zulon
// Define error type
enum ParseError {
    EmptyString,
    InvalidDigit(char),
    Overflow,
}

// Function with | separator
fn parse_int(s: string) -> i32 | ParseError {
    if s.is_empty() {
        throw ParseError::EmptyString;
    }

    let mut result = 0;
    for c in s.chars() {
        if !c.is_digit() {
            throw ParseError::InvalidDigit(c);
        }
        result = result * 10 + (c as i32 - '0' as i32);
    }

    Outcome::Ok(result)
}

// Usage with ?
fn double_input() -> i32 | ParseError {
    let input = read_line()?;  // Assume returns Outcome<string, IoError>
    let value = parse_int(input)?;
    Outcome::Ok(value * 2)
}
```

### Example 2: Error Conversion

```zulon
// Different error types
enum FileError {
    NotFound(string),
    PermissionDenied(string),
}

enum DatabaseError {
    ConnectionFailed,
    QueryFailed(string),
}

// Convert errors
fn load_config(path: string) -> Config | DatabaseError {
    let content = read_file(path)
        .map_err(|e| match e {
            FileError::NotFound(p) => DatabaseError::QueryFailed(format!("File not found: {}", p)),
            FileError::PermissionDenied(p) => DatabaseError::QueryFailed(format!("Permission denied: {}", p)),
        })?;

    parse_config(content)
}
```

### Example 3: Multiple Errors with Effects

```zulon
// Function that can fail and has effects
fn write_log(msg: string) -> unit | IoError | Io {
    let file = open_log_file()?;
    file.write_line(msg)?;
    Outcome::Ok(())
}

// Effects + error
fn handle_request(req: Request) -> Response | HttpError | Io + Database {
    let user = authenticate_user(req)?;  // Can throw HttpError
    let data = fetch_user_data(user.id)?;  // Can throw DatabaseError
    Outcome::Ok(Response::ok(data))
}
```

---

## Testing Strategy

### Unit Tests (40 tests)

#### Lexer Tests (10)
```rust
// crates/zulon-parser/src/lexer/error_handling_tests.rs
#[test]
fn test_throw_keyword() { }
#[test]
fn test_question_mark() { }
#[test]
fn test_pipe_separator() { }
#[test]
fn test_double_pipe_shorthand() { }
#[test]
fn test_throw_in_expression() { }
// ... 5 more
```

#### Parser Tests (15)
```rust
// crates/zulon-parser/src/parser/error_handling_tests.rs
#[test]
fn test_throw_statement() { }
#[test]
fn test_question_mark_in_expression() { }
#[test]
fn test_pipe_separator_simple() { }
#[test]
fn test_pipe_separator_with_effects() { }
#[test]
fn test_double_pipe_shorthand() { }
#[test]
fn test_nested_question_marks() { }
#[test]
fn test_throw_in_match() { }
#[test]
fn test_throw_in_if() { }
// ... 7 more
```

#### Type System Tests (10)
```rust
// crates/zulon-typeck/src/error_handling_tests.rs
#[test]
fn test_throw_type_checking() { }
#[test]
fn test_question_mark_type_checking() { }
#[test]
fn test_error_coercion() { }
#[test]
fn test_incompatible_error_type() { }
#[test]
fn test_throw_in_infallible_function() { }
// ... 5 more
```

#### Code Generation Tests (5)
```rust
// crates/zulon-codegen-llvm/src/error_handling_tests.rs
#[test]
fn test_throw_codegen() { }
#[test]
fn test_question_mark_codegen() { }
#[test]
fn test_outcome_representation() { }
// ... 2 more
```

### Integration Tests (5 programs)

```zulon
// examples/error_handling/01_basic_throw.zl
enum MyError {
    InvalidInput,
}

fn validate(x: i32) -> i32 | MyError {
    if x < 0 {
        throw MyError::InvalidInput;
    }
    Outcome::Ok(x)
}

fn main() {
    match validate(42) {
        Outcome::Ok(v) => println!("Valid: {}", v),
        Outcome::Err(e) => println!("Error: {:?}", e),
    }
}
```

```zulon
// examples/error_handling/02_question_mark.zl
fn divide(a: i32, b: i32) -> i32 | DivideError {
    if b == 0 {
        throw DivideError::Zero;
    }
    Outcome::Ok(a / b)
}

fn calculate() -> i32 | DivideError {
    let x = divide(10, 2)?;
    let y = divide(x, 5)?;
    Outcome::Ok(y)
}

fn main() {
    match calculate() {
        Outcome::Ok(v) => println!("Result: {}", v),
        Outcome::Err(e) => println!("Error: {:?}", e),
    }
}
```

```zulon
// examples/error_handling/03_error_conversion.zl
fn main() -> unit | IoError {
    let content = read_file("config.txt")?;
    let config = parse_config(content)?;
    Outcome::Ok(())
}
```

```zulon
// examples/error_handling/04_multiple_errors.zl
enum ParseError { /* ... */ }
enum IoError { /* ... */ }

fn load_data(path: string) -> Data | ParseError | Io {
    let content = read_file(path)?;
    parse_data(content)
}
```

```zulon
// examples/error_handling/05_advanced_errors.zl
// Comprehensive example with all features
```

---

## Implementation Timeline

### Week 1: Foundation (Day 1-5)

**Day 1-2: Lexer**
- [x] Design document completed
- [ ] Add TokenKind variants
- [ ] Implement lexer modifications
- [ ] Add 10 lexer tests
- [ ] All tests passing

**Day 3-5: Parser**
- [ ] Design AST nodes
- [ ] Implement throw statement parsing
- [ ] Implement ? operator parsing
- [ ] Implement | separator parsing
- [ ] Add 15 parser tests
- [ ] All tests passing

**End of Week 1 Deliverables**:
- ✅ Lexer fully supports error handling syntax
- ✅ Parser fully supports error handling syntax
- ✅ 25 tests passing
- ✅ Documentation updated

### Week 2: Type System & Code Generation (Day 6-10)

**Day 6-8: Type System**
- [ ] Design type checking rules
- [ ] Implement throw statement type checking
- [ ] Implement ? operator type checking
- [ ] Implement | separator type checking
- [ ] Add 10 type system tests
- [ ] All tests passing

**Day 9-10: Code Generation**
- [ ] Design Outcome representation
- [ ] Implement throw statement codegen
- [ ] Implement ? operator codegen
- [ ] Add 5 codegen tests
- [ ] All tests passing

**End of Week 2 Deliverables**:
- ✅ Type system fully supports error handling
- ✅ Code generation fully supports error handling
- ✅ 40 tests passing (total)
- ✅ All integration examples working

### Week 2: Integration & Polish (Day 11-14)

**Day 11-12: Integration**
- [ ] Create 5 example programs
- [ ] Test end-to-end compilation
- [ ] Verify all examples run correctly
- [ ] Performance benchmarking

**Day 13-14: Documentation**
- [ ] Update language reference
- [ ] Write error handling guide
- [ ] Create migration guide from Outcome
- [ ] Update IMPLEMENTATION_PLAN.md

**End of Week 2 Deliverables**:
- ✅ All 5 examples working
- ✅ Complete documentation
- ✅ Performance baseline established
- ✅ Ready for review

---

## Success Criteria

### Functional Requirements
- [x] Design document approved
- [ ] All 40 unit tests passing
- [ ] All 5 integration examples working
- [ ] Zero compiler warnings
- [ ] No regressions in existing tests

### Quality Requirements
- [ ] Code review approved
- [ ] Documentation complete
- [ ] Performance within 5% of manual Outcome handling
- [ ] Error messages clear and helpful

### Integration Requirements
- [ ] Works with existing Outcome<T, E> type
- [ ] Compatible with existing standard library
- [ ] No breaking changes to Phase 1

---

## Open Questions

1. **Should `?` work with `Optional<T>`?**
   - Proposal: Yes, return unit error if None
   - Syntax: `let x = optional()?;`

2. **Should we support automatic error wrapping?**
   - Proposal: No, explicit conversions only
   - Rationale: Clarity over convenience

3. **How to handle multiple error types?**
   - Proposal: Use trait objects or sum types
   - Syntax: `fn f() -> T | dyn Error`

4. **Should we support async error propagation?**
   - Proposal: Yes, same syntax in async functions
   - Implementation: Future iteration

---

## References

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Swift Error Handling](https://docs.swift.org/swift-book/LanguageGuide/ErrorHandling.html)
- [Result Types in Functional Programming](https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/error-handling)

---

**Document Version**: 1.0
**Author**: ZULON Language Team
**Date**: 2026-01-08
**Status**: Ready for Implementation
**Next Review**: End of Week 1 (2026-01-15)
