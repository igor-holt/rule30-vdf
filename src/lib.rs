// =============================================================================
// Rule 30 VDF — Winterfell STARK Implementation
// Schema: ESF-2026-GEN-V3 / Rule30_VDF_Fp_Arithmetization
// Author: Igor Holt
//
// Trace layout (width = 2*N columns):
//   Columns 0..N-1      : spatial cells x_i  (boolean in F_p)
//   Columns N..2*N-1     : intermediate w_i   (OR gate output, boolean in F_p)
//
// AIR constraints (all degree <= 2):
//   AIR-12  P_bool(x_i)  = x_i^2 - x_i                              == 0
//   AIR-12  P_bool(w_i)  = w_i^2 - w_i                              == 0
//   AIR-14  P_or(x,w)    = w_i - (x_i + x_{i+1} - x_i * x_{i+1})   == 0
//   AIR-15  P_xor(x,w,y) = y_i - (x_{i-1} + w_i - 2*x_{i-1}*w_i)  == 0
// =============================================================================

pub mod air;
pub mod prover;
pub mod trace;

pub const DEFAULT_NUM_CELLS: usize = 64;
