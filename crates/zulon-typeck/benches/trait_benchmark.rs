// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Performance benchmarks for the trait system
//!
//! These benchmarks measure the performance of critical trait system operations.

use std::time::Instant;
use zulon_typeck::traits::*;
use zulon_typeck::{GenericParam, Ty};

fn main() {
    println!("ZULON Trait System Performance Benchmarks");
    println!("==========================================\n");

    benchmark_trait_registration();
    benchmark_implementation_lookup();
    benchmark_constraint_solving();
    benchmark_vtable_generation();
    benchmark_large_trait_hierarchy();
    println!("\nAll benchmarks completed!");
}

fn benchmark_trait_registration() {
    println!("Benchmark: Trait Registration");
    println!("------------------------------");

    let mut solver = TraitSolver::new();
    let iterations = 1000;

    let start = Instant::now();
    for i in 0..iterations {
        let id = solver.fresh_trait_id();
        let trait_def = TraitDef::new(format!("Trait_{}", i), id).with_method(
            format!("method_{}", i),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::I32),
        );
        solver.register_trait(trait_def);
    }
    let duration = start.elapsed();

    println!("Registered {} traits in {:?}", iterations, duration);
    println!(
        "Average time per trait: {:.2}μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!();
}

fn benchmark_implementation_lookup() {
    println!("Benchmark: Implementation Lookup");
    println!("------------------------------");

    let mut solver = TraitSolver::new();

    // Create 100 traits
    for i in 0..100 {
        let id = solver.fresh_trait_id();
        let trait_def = TraitDef::new(format!("Trait_{}", i), id);
        solver.register_trait(trait_def);
    }

    // Implement the middle trait for multiple types
    let target_trait_id = TraitId::new(50);
    let target_trait_ref = TraitRef::new(target_trait_id);
    let types = vec![Ty::I32, Ty::I64, Ty::String, Ty::Bool, Ty::F32, Ty::F64];

    for ty in &types {
        let impl_block = TraitImpl::new(ty.clone(), target_trait_ref.clone());
        solver.register_impl(impl_block);
    }

    // Benchmark lookup operations
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = solver.find_impl(&Ty::I32, &target_trait_ref);
        let _ = solver.find_impl(&Ty::String, &target_trait_ref);
    }
    let duration = start.elapsed();

    println!("Performed {} lookups in {:?}", iterations, duration);
    println!(
        "Average time per lookup: {:.2}ns",
        duration.as_nanos() as f64 / iterations as f64
    );
    println!();
}

fn benchmark_constraint_solving() {
    println!("Benchmark: Constraint Solving");
    println!("------------------------------");

    let mut solver = TraitSolver::new();

    // Setup: Create traits and implementations
    for i in 0..100 {
        let id = solver.fresh_trait_id();
        let trait_def = TraitDef::new(format!("Trait_{}", i), id);
        solver.register_trait(trait_def);

        let trait_ref = TraitRef::new(id);
        let impl_block = TraitImpl::new(Ty::I32, trait_ref.clone());
        solver.register_impl(impl_block);
    }

    // Benchmark constraint solving
    let iterations = 1000;
    let start = Instant::now();
    for i in 0..iterations {
        let trait_id = TraitId::new(i);
        let trait_ref = TraitRef::new(trait_id);
        solver.add_constraint(TraitConstraint::Impl {
            ty: Ty::I32,
            trait_ref,
        });
        solver.solve();
    }
    let duration = start.elapsed();

    println!("Solved {} constraints in {:?}", iterations, duration);
    println!(
        "Average time per solve: {:.2}μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!();
}

fn benchmark_vtable_generation() {
    println!("Benchmark: VTable Generation");
    println!("------------------------------");

    let mut solver = TraitSolver::new();

    // Create a trait with multiple methods
    let trait_id = solver.fresh_trait_id();
    let trait_def = TraitDef::new("MultiMethod".to_string(), trait_id)
        .with_method(
            "method1".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::I32),
        )
        .with_method(
            "method2".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::String),
        )
        .with_method(
            "method3".to_string(),
            FunctionSig::new(
                Some(SelfTy::MutRefSelf),
                vec![("x".to_string(), Ty::I32)],
                Ty::Unit,
            ),
        )
        .with_method(
            "method4".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::Bool),
        )
        .with_method(
            "method5".to_string(),
            FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::F64),
        );
    solver.register_trait(trait_def);

    // Implement the trait for multiple types
    let trait_ref = TraitRef::new(trait_id);
    for i in 0..10 {
        let impl_block = TraitImpl::new(Ty::I32, trait_ref.clone())
            .with_method("method1".to_string(), i * 5 + 1)
            .with_method("method2".to_string(), i * 5 + 2)
            .with_method("method3".to_string(), i * 5 + 3)
            .with_method("method4".to_string(), i * 5 + 4)
            .with_method("method5".to_string(), i * 5 + 5);
        solver.register_impl(impl_block);
    }

    // Benchmark vtable generation
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = solver.generate_vtable(&Ty::I32, &trait_ref);
    }
    let duration = start.elapsed();

    println!("Generated {} vtables in {:?}", iterations, duration);
    println!(
        "Average time per vtable: {:.2}ns",
        duration.as_nanos() as f64 / iterations as f64
    );
    println!();
}

fn benchmark_large_trait_hierarchy() {
    println!("Benchmark: Large Trait Hierarchy");
    println!("------------------------------");

    let mut solver = TraitSolver::new();

    // Create a trait hierarchy with inheritance
    let mut trait_ids = Vec::new();
    let hierarchy_depth = 10;
    let traits_per_level = 5;

    for level in 0..hierarchy_depth {
        for i in 0..traits_per_level {
            let id = solver.fresh_trait_id();
            let mut trait_def = TraitDef::new(format!("Level{}_Trait{}", level, i), id);

            // Add super traits from previous level
            if level > 0 {
                let parent_idx = (level - 1) * traits_per_level + (i % traits_per_level);
                let parent_trait = TraitBound::Trait(Ty::TyVar(parent_idx));
                trait_def = trait_def.with_super_trait(parent_trait);
            }

            // Add a method
            trait_def = trait_def.with_method(
                "process".to_string(),
                FunctionSig::new(Some(SelfTy::RefSelf), vec![], Ty::Unit),
            );

            solver.register_trait(trait_def);
            trait_ids.push(id);

            // Implement for multiple types
            let trait_ref = TraitRef::new(id);
            let types = vec![Ty::I32, Ty::I64, Ty::String];
            for ty in &types {
                let impl_block = TraitImpl::new(ty.clone(), trait_ref.clone())
                    .with_method("process".to_string(), 1);
                solver.register_impl(impl_block);
            }
        }
    }

    // Test constraint solving across the hierarchy
    let iterations = 1000;
    let start = Instant::now();
    for i in 0..iterations {
        let trait_id = trait_ids[i % trait_ids.len()];
        let trait_ref = TraitRef::new(trait_id);
        solver.add_constraint(TraitConstraint::Impl {
            ty: Ty::I32,
            trait_ref,
        });
        solver.solve();
    }
    let duration = start.elapsed();

    println!(
        "Hierarchy: {} levels x {} traits = {} total traits",
        hierarchy_depth,
        traits_per_level,
        trait_ids.len()
    );
    println!(
        "Solved {} constraints across hierarchy in {:?}",
        iterations, duration
    );
    println!(
        "Average time per solve: {:.2}μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!();
}
