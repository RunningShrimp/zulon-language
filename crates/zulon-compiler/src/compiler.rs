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
use crate::macro_expander::MacroExpander;

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

        // Compile source to LLVM IR
        self.compile_source(&source, input)?;

        // Try to compile to executable if LLVM tools are available
        let ll_path = input.with_extension("ll");
        match self.compile_ll_to_executable(&ll_path, input) {
            Ok(exe_path) => {
                println!("🎉 Executable created: {}", exe_path.display());
                Ok(exe_path)
            }
            Err(e) => {
                println!("⚠️  Could not create executable: {}", e);
                println!("   LLVM IR is available at: {}", ll_path.display());
                Ok(ll_path)
            }
        }
    }

    /// Compile LLVM IR to executable using llc and clang
    fn compile_ll_to_executable(&self, ll_path: &Path, original_input: &Path) -> CompilerResult<PathBuf> {
        // Generate assembly using llc
        let asm_path = original_input.with_extension("s");

        println!("  🔧 Compiling LLVM IR to assembly...");
        let llc_status = std::process::Command::new("llc")
            .arg(ll_path)
            .arg("-o")
            .arg(&asm_path)
            .output();

        match llc_status {
            Ok(output) if output.status.success() => {
                println!("    ✅ Assembly generated: {}", asm_path.display());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CompilerError::Link(format!("llc failed: {}", stderr)));
            }
            Err(e) => {
                return Err(CompilerError::Link(format!("llc not found: {}", e)));
            }
        }

        // Assemble and link using clang
        let exe_path = if cfg!(target_os = "windows") {
            original_input.with_extension("exe")
        } else {
            original_input.to_path_buf()
        };

        println!("  🔧 Linking executable...");
        let clang_status = std::process::Command::new("clang")
            .arg(&asm_path)
            .arg("-o")
            .arg(&exe_path)
            .output();

        match clang_status {
            Ok(output) if output.status.success() => {
                println!("    ✅ Executable created");
                // Clean up assembly file if not keeping intermediates
                if !self.config.keep_intermediates {
                    let _ = std::fs::remove_file(&asm_path);
                }
                Ok(exe_path)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(CompilerError::Link(format!("clang failed: {}", stderr)))
            }
            Err(e) => {
                Err(CompilerError::Link(format!("clang not found: {}", e)))
            }
        }
    }

    /// Compile ZULON source code
    fn compile_source(&self, source: &str, input_path: &Path) -> CompilerResult<()> {
        println!("🔨 Compiling: {}", input_path.display());

        // Step -1: Inject standard prelude
        let prelude = r#"
// ZULON Standard Prelude - Automatically injected by compiler
extern fn printf(format: &u8, ...) -> i32;
"#;

        let source_with_prelude = format!("{}\n{}", prelude, source);

        // Step 0: Macro expansion
        println!("  [0/8] Macro expansion...");
        let expander = MacroExpander::new();
        let expanded_source = expander.expand_source(&source_with_prelude).map_err(|e| {
            CompilerError::macro_expansion(format!("Macro expansion failed: {}", e))
        })?;

        // Check if any macros were expanded
        if expanded_source != source_with_prelude {
            println!("    ✅ Macros expanded");
        } else {
            println!("    ✅ No macros to expand");
        }

        // Step 1: Lexical analysis
        println!("  [1/8] Lexical analysis...");
        let lexer = Lexer::new(&expanded_source);
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
        println!("  [2/8] Parsing...");
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
        println!("  [3/8] Type checking...");
        let mut typeck = TypeChecker::new();
        typeck.check(&ast).map_err(|e| {
            let error_msg = self.format_typeck_error(&e, input_path);
            CompilerError::type_check(error_msg)
        })?;
        println!("    ✅ Type checked");

        // Step 4: HIR lowering
        println!("  [4/8] HIR lowering...");
        let mut hir_lowerer = SimpleLoweringContext::new();
        let hir_crate = hir_lowerer.lower_ast(&ast)
            .map_err(|e| CompilerError::HirLowering(format!("{:?}", e)))?;
        println!("    ✅ HIR generated ({} items)", hir_crate.items.len());

        // Discover tests and save metadata
        use zulon_hir::test_discovery;
        let tests = test_discovery::discover_tests(&hir_crate);
        if !tests.is_empty() {
            let test_metadata_path = input_path.with_extension("test.json");
            let test_json = serde_json::to_string_pretty(&tests)
                .map_err(|e| CompilerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            std::fs::write(&test_metadata_path, test_json)
                .map_err(|e| CompilerError::Io(e))?;
            println!("    ✅ Discovered {} tests → {}", tests.len(), test_metadata_path.display());
        }

        // Step 5: MIR lowering
        println!("  [5/8] MIR lowering...");
        let mut mir_lowerer = MirLoweringContext::new();
        let mir_body = mir_lowerer.lower_crate(&hir_crate)
            .map_err(|e| CompilerError::MirLowering(format!("{:?}", e)))?;
        println!("    ✅ MIR generated ({} functions)", mir_body.functions.len());

        // Step 6: LIR lowering
        println!("  [6/8] LIR lowering...");
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
        println!("  [7/8] Generating LLVM IR...");
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

    /// Format type check errors with helpful context using the diagnostic system
    fn format_typeck_error(&self, error: &zulon_typeck::TypeError, file_path: &Path) -> String {
        // Read source file for diagnostics
        let source = std::fs::read_to_string(file_path)
            .unwrap_or_else(|_| "".to_string());

        // Convert TypeError to Diagnostic and display with context
        let diagnostic = error.to_diagnostic(&source);

        // Use colors if terminal supports it
        let use_colors = std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stderr);

        diagnostic.display_with_context(&source, use_colors)
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
