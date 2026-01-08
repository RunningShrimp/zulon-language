// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Compiler implementation

use std::path::{Path, PathBuf};
use zulon_parser::{Lexer, Parser};
use zulon_parser::ast::{ItemKind, Type as AstType};
use zulon_typeck::TypeChecker;
use zulon_hir::SimpleLoweringContext;
use zulon_mir::MirLoweringContext;
use zulon_lir::{LirLoweringContext, LirExternal, LirTy};
use zulon_codegen_llvm::CodeGenerator;

use crate::error::{CompilerError, Result as CompilerResult};

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Output file path
    pub output: Option<PathBuf>,
    /// Keep intermediate files
    pub keep_intermediates: bool,
    /// Target triple
    pub target: Option<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: 2,
            output: None,
            keep_intermediates: false,
            target: None,
        }
    }
}

/// ZULON compiler
pub struct Compiler {
    #[allow(dead_code)]
    config: CompilerConfig,
}

impl Compiler {
    /// Create a new compiler
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a ZULON source file to an executable
    pub fn compile_file(&self, input: &Path) -> CompilerResult<PathBuf> {
        // Read source file
        let source = std::fs::read_to_string(input)
            .map_err(|e| CompilerError::Io(e))?;

        // Compile source (generates .ll file)
        self.compile_source(&source, input)?;

        // Return the .ll file path for now
        Ok(input.with_extension("ll"))
    }

    /// Compile ZULON source code
    fn compile_source(&self, source: &str, input_path: &Path) -> CompilerResult<()> {
        println!("🔨 Compiling: {}", input_path.display());

        // Step 1: Lexical analysis
        println!("  [1/7] Lexical analysis...");
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.lex_all();

        if !lex_errors.is_empty() {
            for err in &lex_errors {
                eprintln!("    Lexical error: {:?}", err);
            }
            return Err(CompilerError::lexical(format!(
                "{} lexical errors",
                lex_errors.len()
            )));
        }

        // Filter out comment tokens (they're not needed for parsing)
        let tokens: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, zulon_parser::TokenKind::Comment))
            .collect();

        println!("    ✅ {} tokens generated", tokens.len());

        // Step 2: Parsing
        println!("  [2/7] Parsing...");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| {
            let error_msg = self.format_parse_error(&e, input_path);
            CompilerError::parse(error_msg)
        })?;
        println!("    ✅ AST parsed");

        // Extract extern function declarations
        let extern_functions = self.extract_extern_functions(&ast);
        if !extern_functions.is_empty() {
            println!("    📦 Found {} extern function(s)", extern_functions.len());
        }

        // Step 3: Type checking
        println!("  [3/7] Type checking...");
        let mut typeck = TypeChecker::new();
        typeck.check(&ast).map_err(|e| {
            let error_msg = self.format_typeck_error(&e, input_path);
            CompilerError::type_check(error_msg)
        })?;
        println!("    ✅ Type checked");

        // Step 4: HIR lowering
        println!("  [4/7] HIR lowering...");
        let mut hir_lowerer = SimpleLoweringContext::new();
        let hir_crate = hir_lowerer.lower_ast(&ast)
            .map_err(|e| CompilerError::HirLowering(format!("{:?}", e)))?;
        println!("    ✅ HIR generated ({} items)", hir_crate.items.len());

        // Step 5: MIR lowering
        println!("  [5/7] MIR lowering...");
        let mut mir_lowerer = MirLoweringContext::new();
        let mir_body = mir_lowerer.lower_crate(&hir_crate)
            .map_err(|e| CompilerError::MirLowering(format!("{:?}", e)))?;
        println!("    ✅ MIR generated ({} functions)", mir_body.functions.len());

        // Step 6: LIR lowering
        println!("  [6/7] LIR lowering...");
        let mut lir_lowerer = LirLoweringContext::new();
        let mut lir_body = lir_lowerer.lower_body(&mir_body)
            .map_err(|e| CompilerError::LirLowering(format!("{:?}", e)))?;
        println!("    ✅ LIR generated ({} functions)", lir_body.functions.len());

        // Add extern functions from source code
        for extern_func in extern_functions {
            lir_body.push_external(extern_func);
        }
        println!("    ✅ Added {} extern functions", lir_body.externals.len());

        // Step 7: Generate LLVM IR
        println!("  [7/7] Generating LLVM IR...");
        let output_path = input_path.with_extension("ll");
        let output_file = std::fs::File::create(&output_path)
            .map_err(|e| CompilerError::Io(e))?;

        // Generate real LLVM IR from LIR
        let mut codegen = CodeGenerator::new(output_file);
        codegen.generate_module_with_externals(
            &lir_body.functions,
            &lir_body.externals,
        ).map_err(|e| CompilerError::CodeGen(format!("{:?}", e)))?;

        println!("    ✅ Generated LLVM IR: {}", output_path.display());
        println!();
        println!("✅ Compilation successful!");
        println!("   LLVM IR saved to: {}", output_path.display());
        println!("   To compile to executable:");
        println!("     llc {}.ll -o {}.s", input_path.display(), input_path.display());
        println!("     clang {}.s -o {}", input_path.display(), input_path.display());

        Ok(())
    }

    /// Extract extern function declarations from the AST
    fn extract_extern_functions(&self, ast: &zulon_parser::ast::Ast) -> Vec<LirExternal> {
        let mut externs = Vec::new();

        for item in &ast.items {
            if let ItemKind::ExternFunction(func) = &item.kind {
                // Convert parameter types
                let param_types: Vec<LirTy> = func.params.iter()
                    .filter_map(|p| p.type_annotation.as_ref())
                    .map(|ty| self.ast_type_to_lir_type(ty))
                    .collect();

                // Get return type
                let return_type = func.return_type.as_ref()
                    .map(|ty| self.ast_type_to_lir_type(ty))
                    .unwrap_or(LirTy::Unit);

                externs.push(LirExternal {
                    name: func.name.name.clone(),
                    param_types,
                    return_type,
                });
            }
        }

        externs
    }

    /// Convert AST type to LIR type (simplified version)
    fn ast_type_to_lir_type(&self, ty: &AstType) -> LirTy {
        match ty {
            AstType::Simple(ident) => {
                match ident.name.as_str() {
                    "i32" => LirTy::I32,
                    "i64" => LirTy::I64,
                    "u8" => LirTy::U8,
                    "u32" => LirTy::U32,
                    "u64" => LirTy::U64,
                    "f32" => LirTy::F32,
                    "f64" => LirTy::F64,
                    "bool" => LirTy::Bool,
                    "str" | "String" => LirTy::Ptr(Box::new(LirTy::U8)),
                    _ => LirTy::I32, // Default to i32 for unknown types
                }
            }
            AstType::Ref(base, _mut) => {
                LirTy::Ptr(Box::new(self.ast_type_to_lir_type(base)))
            }
            _ => LirTy::I32, // Default to i32 for complex types
        }
    }

    /// Format parse errors with helpful context
    fn format_parse_error(&self, error: &zulon_parser::ParseError, file_path: &Path) -> String {
        use zulon_parser::ParseError;
        use std::fmt::Write;

        let mut msg = String::new();

        match error {
            ParseError::UnexpectedToken { expected, found, span } => {
                writeln!(msg, "Parse error: {}", self.format_location(span, file_path)).unwrap_or(());
                writeln!(msg, "  Expected: {}", expected).unwrap_or(());
                writeln!(msg, "  Found: {:?}", found).unwrap_or(());

                // Add helpful suggestions
                if expected.contains("Semicolon") {
                    writeln!(msg).unwrap_or(());
                    writeln!(msg, "  💡 Hint: Add a semicolon (;) after the previous statement").unwrap_or(());
                    writeln!(msg, "  Example:").unwrap_or(());
                    writeln!(msg, "    let x = 10;  ← Add semicolon here").unwrap_or(());
                }
            }
            ParseError::UnexpectedEof { span } => {
                writeln!(msg, "Parse error: {}", self.format_location(span, file_path)).unwrap_or(());
                writeln!(msg, "  Unexpected end of file").unwrap_or(());
                writeln!(msg).unwrap_or(());
                writeln!(msg, "  💡 Hint: Check that all braces, parentheses, and brackets are properly closed").unwrap_or(());
            }
            ParseError::InvalidSyntax { message, span } => {
                writeln!(msg, "Parse error: {}", self.format_location(span, file_path)).unwrap_or(());
                writeln!(msg, "  {}", message).unwrap_or(());
            }
            ParseError::ModuleError { source } => {
                return self.format_parse_error(source, file_path);
            }
        }

        msg
    }

    /// Format type check errors with helpful context
    fn format_typeck_error(&self, error: &zulon_typeck::TypeError, file_path: &Path) -> String {
        use zulon_typeck::TypeError;
        use std::fmt::Write;

        let mut msg = String::new();

        match error {
            TypeError::UndefinedVariable { name, span } => {
                writeln!(msg, "Type error: {}", self.format_location_span(span, file_path)).unwrap_or(());
                writeln!(msg, "  Undefined variable: '{}'", name).unwrap_or(());
                writeln!(msg).unwrap_or(());
                writeln!(msg, "  💡 Hint: Check that the variable is spelled correctly").unwrap_or(());
                writeln!(msg, "  💡 Hint: Make sure the variable is declared before use").unwrap_or(());
            }
            _ => {
                writeln!(msg, "Type error: {:?}", error).unwrap_or(());
            }
        }

        msg
    }

    /// Format error location with file context (for generic spans)
    fn format_location_span(&self, span: &zulon_parser::Span, file_path: &Path) -> String {
        format!("{}:{}:{} to {}:{}",
            file_path.display(),
            span.start.line,
            span.start.column,
            span.end.line,
            span.end.column
        )
    }

    /// Format error location with file context
    fn format_location(&self, span: &zulon_parser::Span, file_path: &Path) -> String {
        format!("{}:{}:{} to {}:{}",
            file_path.display(),
            span.start.line,
            span.start.column,
            span.end.line,
            span.end.column
        )
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}
