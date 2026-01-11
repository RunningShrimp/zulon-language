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
use zulon_codegen_llvm::{CodeGenerator, StructLayout};
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
        // Use a different name for the executable to avoid overwriting the source file
        let exe_path = if let Some(ref output) = self.config.output {
            output.clone()
        } else if cfg!(target_os = "windows") {
            original_input.with_extension("exe")
        } else {
            // On Unix-like systems, add no extension to avoid overwriting .zl files
            // The executable will have the same base name as the input
            let base_name = original_input.file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("aout"));
            
            // Build the path in the same directory as the source
            let exe_name = base_name.to_os_string().to_owned();
            let parent_dir = original_input.parent();
            
            match parent_dir {
                Some(dir) => dir.join(exe_name),
                None => PathBuf::from(exe_name),
            }
        };

        // Compile runtime library
        println!("  🔧 Compiling runtime library...");
        let runtime_dir = std::path::Path::new("runtime");
        let runtime_c = runtime_dir.join("zulon_runtime.c");
        let runtime_o = runtime_dir.join("zulon_runtime.o");

        // Check if runtime exists
        if runtime_c.exists() {
            // Compile runtime C code to object file
            let gcc_status = std::process::Command::new("clang")
                .arg("-c")
                .arg(&runtime_c)
                .arg("-o")
                .arg(&runtime_o)
                .arg("-O2")
                .output();

            match gcc_status {
                Ok(output) if output.status.success() => {
                    println!("    ✅ Runtime library compiled");
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(CompilerError::Link(format!("Runtime compilation failed: {}", stderr)));
                }
                Err(e) => {
                    return Err(CompilerError::Link(format!("clang not found for runtime: {}", e)));
                }
            }
        } else {
            println!("    ⚠️  Runtime library not found, template strings may not work");
        }

        println!("  🔧 Linking executable...");
        let mut clang_cmd = std::process::Command::new("clang");
        clang_cmd
            .arg(&asm_path);

        // Link runtime library if it was compiled
        if runtime_o.exists() {
            clang_cmd.arg(&runtime_o);
        }

        // Link cargo-built runtime libraries (async I/O, async runtime, etc.)
        // Find the cargo build output directory
        let target_dir = std::path::Path::new("target");
        let debug_dir = target_dir.join("debug");
        let mut runtime_lib_dir = None;

        // Try to find the build output directory with runtime libraries
        if let Ok(entries) = std::fs::read_dir(&debug_dir.join("build")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let out_dir = path.join("out");
                    if out_dir.exists() {
                        // Check if this contains our runtime libraries
                        let async_io_lib = out_dir.join("libzulon_async_io.a");
                        if async_io_lib.exists() {
                            runtime_lib_dir = Some(out_dir);
                            break;
                        }
                    }
                }
            }
        }

        // If we found the runtime library directory, link all the libraries
        if let Some(lib_dir) = runtime_lib_dir {
            println!("    🔗 Linking runtime libraries from: {}", lib_dir.display());

            // Add library search path
            clang_cmd.arg("-L").arg(&lib_dir);

            // Link all runtime libraries in the correct dependency order
            let runtime_libs = vec![
                "zulon_async_io",
                "zulon_async_runtime",
                "zulon_event_loop",
                "zulon_coroutine",
                "zulon_scheduler",
                "zulon_future",
                "zulon_channel",
                "zulon_select",
                "zulon_time",
                "zulon_string",
                "zulon_entry",
            ];

            for lib in runtime_libs {
                clang_cmd.arg(format!("-l{}", lib));
            }
        } else {
            println!("    ⚠️  Runtime libraries not found, async I/O may not work");
        }

        clang_cmd
            .arg("-o")
            .arg(&exe_path);

        let clang_status = clang_cmd.output();

        match clang_status {
            Ok(output) if output.status.success() => {
                println!("    ✅ Executable created");
                // Clean up assembly file and runtime object if not keeping intermediates
                if !self.config.keep_intermediates {
                    let _ = std::fs::remove_file(&asm_path);
                    let _ = std::fs::remove_file(&runtime_o);
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
        // Note: printf and scanf are auto-injected in extract_extern_functions
        // So we don't need to declare them here
        let source_with_prelude = source;

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
        use zulon_hir::test_main_gen;
        
        let tests = test_discovery::discover_tests(&hir_crate);
        if !tests.is_empty() {
            // Save test metadata
            let test_metadata_path = input_path.with_extension("test.json");
            let test_json = serde_json::to_string_pretty(&tests)
                .map_err(|e| CompilerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            std::fs::write(&test_metadata_path, test_json)
                .map_err(|e| CompilerError::Io(e))?;
            println!("    ✅ Discovered {} tests → {}", tests.len(), test_metadata_path.display());
            
            // Generate test main file
            let test_main_path = input_path.with_extension("test_main.zl");
            let test_main_source = test_main_gen::generate_test_main_source(&tests);
            std::fs::write(&test_main_path, test_main_source)
                .map_err(|e| CompilerError::Io(e))?;
            println!("    ✅ Generated test main → {}", test_main_path.display());
            println!("    💡 Compile test main with your test file");
        }

        // Step 5: MIR lowering
        println!("  [5/9] MIR lowering...");
        let mut mir_lowerer = MirLoweringContext::new();
        let mut mir_body = mir_lowerer.lower_crate(&hir_crate)
            .map_err(|e| CompilerError::MirLowering(format!("{:?}", e)))?;
        println!("    ✅ MIR generated ({} functions)", mir_body.functions.len());

        // Step 5.5: Async transformation (convert async functions to state machines)
        println!("  [5.5/9] Async transformation...");
        use zulon_mir::async_transform;

        // Count async functions before transformation
        let async_count = mir_body.functions.iter()
            .filter(|f| f.is_async)
            .count();

        if async_count > 0 {
            println!("    🔄 Found {} async function(s), applying state machine transformation...", async_count);
            match async_transform::transform_async_functions(mir_body) {
                Ok(transformed_body) => {
                    mir_body = transformed_body;
                    println!("    ✅ Transformed {} async function(s) to state machines", async_count);
                }
                Err(e) => {
                    println!("    ⚠️  Async transformation failed: {:?}", e);
                    println!("    💡 Continuing without async transformation...");
                    // Recreate mir_body if transformation failed
                    let mut mir_lowerer = MirLoweringContext::new();
                    mir_body = mir_lowerer.lower_crate(&hir_crate)
                        .map_err(|e| CompilerError::MirLowering(format!("{:?}", e)))?;
                }
            }
        } else {
            println!("    ✅ No async functions found");
        }

        // Step 6: LIR lowering
        println!("  [6/9] LIR lowering...");
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
        println!("  [7/9] Generating LLVM IR...");
        let output_path = input_path.with_extension("ll");
        let output_file = std::fs::File::create(&output_path)
            .map_err(|e| CompilerError::Io(e))?;

        // Generate real LLVM IR from LIR
        let mut codegen = CodeGenerator::new(output_file);

        // Register struct types from LIR before generating module
        // We need to scan for struct types and register them
        use std::collections::HashSet;
        let mut registered_structs = HashSet::new();

        // Helper function to recursively register struct types
        // This handles nested pointer types by unwrapping them
        fn register_struct_recursive(
            ty: &LirTy,
            registered_structs: &mut HashSet<String>,
            codegen: &mut CodeGenerator<std::fs::File>,
        ) {
            // First, recursively check pointer types by unwrapping them
            let mut inner_ty = ty;
            while let LirTy::Ptr(inner) = inner_ty {
                inner_ty = inner;
            }

            // Now check if we have a struct type at the core
            if let LirTy::Struct { name, fields, .. } = inner_ty {
                if !registered_structs.contains(name) {
                    let mut layout = StructLayout::new(name.clone());
                    for (i, field_ty) in fields.iter().enumerate() {
                        let field_name = format!("field{}", i);
                        let _ = layout.add_field(field_name, field_ty.clone());
                    }
                    layout.finalize();
                    codegen.register_struct(layout);
                    registered_structs.insert(name.clone());
                }
            }
        }

        for func in &lir_body.functions {
            // Scan all instructions
            for (_block_id, block) in &func.blocks {
                for instr in &block.instructions {
                    match instr {
                        zulon_lir::LirInstruction::Alloca(a) => {
                            register_struct_recursive(&a.ty, &mut registered_structs, &mut codegen);
                        }
                        zulon_lir::LirInstruction::Const { ty, .. } => {
                            register_struct_recursive(ty, &mut registered_structs, &mut codegen);
                        }
                        zulon_lir::LirInstruction::Load { ty, .. } => {
                            register_struct_recursive(ty, &mut registered_structs, &mut codegen);
                        }
                        zulon_lir::LirInstruction::Store { ty, .. } => {
                            register_struct_recursive(ty, &mut registered_structs, &mut codegen);
                        }
                        zulon_lir::LirInstruction::BinaryOp { ty, .. } => {
                            register_struct_recursive(ty, &mut registered_structs, &mut codegen);
                        }
                        zulon_lir::LirInstruction::Call { return_type, .. } => {
                            register_struct_recursive(return_type, &mut registered_structs, &mut codegen);
                        }
                        _ => {}
                    }
                }

                // Also check terminators
                if let Some(terminator) = &block.terminator {
                    match terminator {
                        zulon_lir::LirTerminator::Return(_) => {},
                        _ => {}
                    }
                }
            }

            // Check parameter and return types
            for param_ty in &func.param_types {
                register_struct_recursive(param_ty, &mut registered_structs, &mut codegen);
            }
            register_struct_recursive(&func.return_type, &mut registered_structs, &mut codegen);
        }

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

        // First, collect explicitly declared extern functions from source code
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

                // Mark known variadic C functions
                let is_variadice = matches!(func.name.name.as_str(), "printf" | "scanf");

                externs.push(LirExternal {
                    name: func.name.name.clone(),
                    param_types,
                    return_type,
                    variadic: is_variadice,
                });
            }
        }

        // CRITICAL FIX: Auto-inject common C standard library functions
        // These are used implicitly by ZULON programs (e.g., printf for debugging)
        // This allows users to call printf() without explicit extern declarations

        // Check if printf is already declared
        let has_printf = externs.iter().any(|e| e.name == "printf");
        if !has_printf {
            // Inject printf declaration: extern fn printf(format: *u8, ...) -> i32
            externs.push(LirExternal {
                name: "printf".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::U8))], // format: *u8
                return_type: LirTy::I32,
                variadic: true, // printf accepts variable arguments
            });
        }

        // Check if scanf is needed (could check function calls, but inject for now)
        let has_scanf = externs.iter().any(|e| e.name == "scanf");
        if !has_scanf {
            // Inject scanf declaration: extern fn scanf(format: *u8, ...) -> i32
            externs.push(LirExternal {
                name: "scanf".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::U8))], // format: *u8
                return_type: LirTy::I32,
                variadic: true, // scanf accepts variable arguments
            });
        }

        // Inject string_concat for template string interpolation
        // This is used by the compiler to implement template string interpolation
        let has_string_concat = externs.iter().any(|e| e.name == "string_concat");
        if !has_string_concat {
            // Inject string_concat declaration: extern fn string_concat(str1: *u8, str2: *u8) -> *u8
            externs.push(LirExternal {
                name: "string_concat".to_string(),
                param_types: vec![
                    LirTy::Ptr(Box::new(LirTy::U8)), // str1: *u8
                    LirTy::Ptr(Box::new(LirTy::U8)), // str2: *u8
                ],
                return_type: LirTy::Ptr(Box::new(LirTy::U8)), // returns *u8
                variadic: false,
            });
        }

        // Phase 2.2: Inject async scheduler extern functions
        // These are used by ZULON's async runtime for non-blocking operations
        println!("  [DEBUG] Checking async_scheduler externs, current count: {}", externs.len());

        // Inject async_scheduler_create: extern fn async_scheduler_create() -> *AsyncScheduler
        let has_async_scheduler_create = externs.iter().any(|e| e.name == "async_scheduler_create");
        if !has_async_scheduler_create {
            println!("  [DEBUG] Injecting async_scheduler_create");
            externs.push(LirExternal {
                name: "async_scheduler_create".to_string(),
                param_types: vec![],
                return_type: LirTy::Ptr(Box::new(LirTy::I8)), // *AsyncScheduler (opaque pointer)
                variadic: false,
            });
        }

        // Inject async_scheduler_run: extern fn async_scheduler_run(scheduler: *AsyncScheduler) -> i32
        let has_async_scheduler_run = externs.iter().any(|e| e.name == "async_scheduler_run");
        if !has_async_scheduler_run {
            externs.push(LirExternal {
                name: "async_scheduler_run".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // scheduler: *AsyncScheduler
                return_type: LirTy::I32,
                variadic: false,
            });
        }

        // Inject async_scheduler_stop: extern fn async_scheduler_stop(scheduler: *AsyncScheduler)
        let has_async_scheduler_stop = externs.iter().any(|e| e.name == "async_scheduler_stop");
        if !has_async_scheduler_stop {
            externs.push(LirExternal {
                name: "async_scheduler_stop".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // scheduler: *AsyncScheduler
                return_type: LirTy::Unit,
                variadic: false,
            });
        }

        // Inject async_scheduler_destroy: extern fn async_scheduler_destroy(scheduler: *AsyncScheduler)
        let has_async_scheduler_destroy = externs.iter().any(|e| e.name == "async_scheduler_destroy");
        if !has_async_scheduler_destroy {
            externs.push(LirExternal {
                name: "async_scheduler_destroy".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // scheduler: *AsyncScheduler
                return_type: LirTy::Unit,
                variadic: false,
            });
        }

        // Inject async_sleep: extern fn async_sleep(scheduler: *AsyncScheduler, duration_ms: i64) -> i32
        let has_async_sleep = externs.iter().any(|e| e.name == "async_sleep");
        if !has_async_sleep {
            externs.push(LirExternal {
                name: "async_sleep".to_string(),
                param_types: vec![
                    LirTy::Ptr(Box::new(LirTy::I8)), // scheduler: *AsyncScheduler
                    LirTy::I64,                       // duration_ms: i64
                ],
                return_type: LirTy::I32,
                variadic: false,
            });
        }

        // Inject async_scheduler_get_event_loop: extern fn async_scheduler_get_event_loop(scheduler: *AsyncScheduler) -> *EventLoop
        let has_async_scheduler_get_event_loop = externs.iter().any(|e| e.name == "async_scheduler_get_event_loop");
        if !has_async_scheduler_get_event_loop {
            externs.push(LirExternal {
                name: "async_scheduler_get_event_loop".to_string(),
                param_types: vec![LirTy::Ptr(Box::new(LirTy::I8))], // scheduler: *AsyncScheduler
                return_type: LirTy::Ptr(Box::new(LirTy::I8)), // *EventLoop (opaque pointer)
                variadic: false,
            });
        }

        // Inject event_loop_add_timer: extern fn event_loop_add_timer(loop: *EventLoop, delay_ms: i64, callback: *u8, data: *u8) -> i32
        let has_event_loop_add_timer = externs.iter().any(|e| e.name == "event_loop_add_timer");
        if !has_event_loop_add_timer {
            externs.push(LirExternal {
                name: "event_loop_add_timer".to_string(),
                param_types: vec![
                    LirTy::Ptr(Box::new(LirTy::I8)), // loop: *EventLoop
                    LirTy::I64,                       // delay_ms: i64
                    LirTy::Ptr(Box::new(LirTy::U8)), // callback: *u8 (function pointer)
                    LirTy::Ptr(Box::new(LirTy::U8)), // data: *u8
                ],
                return_type: LirTy::I32,
                variadic: false,
            });
        }

        eprintln!("DEBUG: extract_extern_functions returning {} externs", externs.len());
        for (i, ext) in externs.iter().enumerate() {
            eprintln!("DEBUG:   [{}] {} -> {:?}", i, ext.name, ext.return_type);
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
