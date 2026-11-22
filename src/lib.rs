// =============================================================================
// Rule 30 VDF — Winterfell STARK Implementation
// Schema: ESF-2026-GEN-V3 / Rule30_VDF_Fp_Arithmetization
// Author: Igor Holt
//
// Trace layout (width = 2*N + 1 columns):
//   Columns 0..N-1      : spatial cells x_i  (boolean in F_p)
//   Columns N..2*N-1    : intermediate w_i   (OR gate output, boolean in F_p)
//   Column  2*N         : row index t        (0, 1, ..., T-1), degree anchor
//
// AIR constraints:
//   AIR-12  P_bool(x_i)  = x_i^2 - x_i                              == 0
//   AIR-12  P_bool(w_i)  = w_i^2 - w_i                              == 0
//   AIR-14  P_or(x,w)    = w_i - (x_i + x_{i+1} - x_i * x_{i+1})   == 0
//   AIR-15  P_xor(x,w,y) = y_i - (x_{i-1} + w_i - 2*x_{i-1}*w_i)  == 0
//   row     t' - (t + 1) = 0                                         (degree 1)
// =============================================================================

//! Verifiable delay function (VDF) on the Rule 30 cellular automaton, with
//! STARK proofs generated and checked by
//! [Winterfell](https://github.com/facebook/winterfell).
//!
//! The delay is `T - 1` sequential Rule 30 steps on a periodic ring of `N`
//! cells. [`trace::build_trace`] does that work and returns the execution
//! trace, [`prover::Rule30Prover`] turns the trace into a proof, and
//! [`prover::verify_proof`] checks the proof against the public input and
//! output without replaying the delay.
//!
//! This is an unaudited research prototype. The proofs are succinct but not
//! zero-knowledge, and sequentiality rests on the conjecture that Rule 30 has
//! no shortcut. The README lists the assumptions and limitations.
//!
//! # Example
//!
//! ```
//! use rule30_vdf::air::Rule30PublicInputs;
//! use rule30_vdf::prover::{default_proof_options, verify_proof, Rule30Prover};
//! use rule30_vdf::trace::build_trace;
//! use winterfell::math::fields::f128::BaseElement;
//!
//! // Seed: a ring of 64 cells with the centre bit set.
//! let mut init = vec![0u8; rule30_vdf::DEFAULT_NUM_CELLS];
//! init[rule30_vdf::DEFAULT_NUM_CELLS / 2] = 1;
//!
//! // The sequential work. A trace of 1,024 rows holds the seed in row 0 and
//! // attests to 1,023 Rule 30 steps. The row count must be a power of two.
//! let (trace, final_state) = build_trace(&init, 1024);
//!
//! let pub_inputs = Rule30PublicInputs {
//!     init_state: init.iter().map(|&bit| BaseElement::from(bit)).collect(),
//!     final_state,
//!     num_cells: init.len(),
//! };
//!
//! let proof = Rule30Prover::new(default_proof_options(), pub_inputs.clone())
//!     .prove_trace(trace)
//!     .expect("proof generation failed");
//!
//! verify_proof(proof, pub_inputs).expect("proof verification failed");
//! ```

pub mod air;
pub mod prover;
pub mod trace;

/// Ring size used by the CLI when `RULE30_CELLS` is not set.
pub const DEFAULT_NUM_CELLS: usize = 64;
