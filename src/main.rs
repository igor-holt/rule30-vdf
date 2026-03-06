// =============================================================================
// Rule 30 VDF — MVP Entry Point
// Schema: ESF-2026-GEN-V3 / Rule30_VDF_Fp_Arithmetization
// Author: Igor Holt
//
// Generates a STARK proof that T sequential steps of Rule 30 were computed
// correctly over N cells in F_p, using Winterfell.
// =============================================================================

use std::time::Instant;
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;

use rule30_vdf::air::Rule30PublicInputs;
use rule30_vdf::prover::{default_proof_options, verify_proof, Rule30Prover};
use rule30_vdf::trace::{build_trace, rule30_plain};
use rule30_vdf::DEFAULT_NUM_CELLS;

fn main() {
    let num_cells: usize = std::env::var("RULE30_CELLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_NUM_CELLS);

    // VDF depth T — must be power of 2
    let num_steps: usize = std::env::var("RULE30_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1024);

    assert!(
        num_steps.is_power_of_two(),
        "RULE30_STEPS must be a power of 2, got {num_steps}"
    );

    println!("=== Rule 30 VDF — STARK Proof (ESF-2026-GEN-V3) ===");
    println!("  Cells (N)     : {num_cells}");
    println!("  Steps (T)     : {num_steps}");
    println!("  Trace width   : {} columns", 2 * num_cells);
    println!("  Constraints   : {} (4*N, all degree 2)", 4 * num_cells);
    println!();

    // Initial state: single 1-bit at center (classic Rule 30 seed)
    let mut init = vec![0u8; num_cells];
    init[num_cells / 2] = 1;

    // --- Compute trace (the sequential VDF work) ---
    println!("[1/4] Computing Rule 30 execution trace...");
    let t0 = Instant::now();
    let (trace, final_state) = build_trace(&init, num_steps);
    let trace_time = t0.elapsed();
    println!("      Trace computed in {:.3}s", trace_time.as_secs_f64());

    // Cross-check against plain computation
    let expected = rule30_plain(&init, num_steps - 1);
    let expected_fe: Vec<BaseElement> = expected
        .iter()
        .map(|&b| if b != 0 { BaseElement::ONE } else { BaseElement::ZERO })
        .collect();
    assert_eq!(
        final_state, expected_fe,
        "Trace final state does not match plain Rule 30 computation"
    );
    println!("      Cross-check: PASS");

    // --- Generate STARK proof ---
    let init_fe: Vec<BaseElement> = init
        .iter()
        .map(|&b| if b != 0 { BaseElement::ONE } else { BaseElement::ZERO })
        .collect();

    let pub_inputs = Rule30PublicInputs {
        init_state: init_fe,
        final_state: final_state.clone(),
        num_cells,
    };

    let options = default_proof_options();
    let prover = Rule30Prover::new(options, pub_inputs.clone());

    println!("[2/4] Generating STARK proof...");
    let t1 = Instant::now();
    let proof = prover
        .prove_trace(trace)
        .expect("proof generation failed");
    let prove_time = t1.elapsed();
    println!("      Proof generated in {:.3}s", prove_time.as_secs_f64());
    println!("      Proof size: {} bytes", proof.to_bytes().len());

    // --- Verify proof ---
    println!("[3/4] Verifying STARK proof...");
    let t2 = Instant::now();
    verify_proof(proof, pub_inputs).expect("proof verification failed");
    let verify_time = t2.elapsed();
    println!("      Verified in {:.3}s", verify_time.as_secs_f64());

    // --- Summary ---
    println!("[4/4] Summary");
    println!("      Sequential work (T={num_steps}): {:.3}s", trace_time.as_secs_f64());
    println!("      Proof generation             : {:.3}s", prove_time.as_secs_f64());
    println!("      Verification                 : {:.3}s", verify_time.as_secs_f64());
    println!();
    println!("  Final state hash (first 8 cells):");
    print!("    ");
    for fe in final_state.iter().take(8) {
        let val: u64 = if *fe == BaseElement::ONE { 1 } else { 0 };
        print!("{val} ");
    }
    println!();
    println!();
    println!("=== VDF MVP COMPLETE — Ready for CaaS Arbitration ===");
}
