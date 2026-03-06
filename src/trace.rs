// =============================================================================
// Execution trace builder for Rule 30 VDF.
// Computes T sequential steps of Rule 30 over N cells in F_p.
// =============================================================================

use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;
use winterfell::TraceTable;

/// Build the full execution trace for `num_steps` iterations of Rule 30.
///
/// `init`: initial boolean state (len = num_cells).
/// `num_steps`: must be a power of 2.
///
/// Returns (trace, final_state).
pub fn build_trace(
    init: &[u8],
    num_steps: usize,
) -> (TraceTable<BaseElement>, Vec<BaseElement>) {
    let n = init.len();
    let trace_width = 2 * n; // x columns + w columns
    let mut trace = TraceTable::new(trace_width, num_steps);

    let one = BaseElement::ONE;
    let zero = BaseElement::ZERO;
    let two = BaseElement::from(2u32);

    let to_fe = |b: u8| -> BaseElement {
        if b != 0 { one } else { zero }
    };

    trace.fill(
        // Init closure: called once for row 0
        |state: &mut [BaseElement]| {
            for i in 0..n {
                state[i] = to_fe(init[i]);
            }
            // w_i = OR(x_i, x_{i+1}) for row 0
            for i in 0..n {
                let x_i = state[i];
                let x_right = state[(i + 1) % n];
                state[n + i] = x_i + x_right - x_i * x_right;
            }
        },
        // Update closure: called for rows 1..num_steps-1.
        // `state` contains the PREVIOUS row; we modify it in-place to become the CURRENT row.
        |_step, state: &mut [BaseElement]| {
            // Snapshot previous x values before overwriting
            let x_prev: Vec<BaseElement> = state[..n].to_vec();

            // Previous w values are already consistent (computed for x_prev)
            let w_prev: Vec<BaseElement> = state[n..2 * n].to_vec();

            // Rule 30 transition: next_x_i = XOR(x_{i-1}, w_i)
            //   = x_{i-1} + w_i - 2 * x_{i-1} * w_i
            for i in 0..n {
                let x_left = x_prev[(i + n - 1) % n];
                state[i] = x_left + w_prev[i] - two * x_left * w_prev[i];
            }

            // Recompute w for the new x (this row's OR constraint must hold)
            for i in 0..n {
                let x_i = state[i];
                let x_right = state[(i + 1) % n];
                state[n + i] = x_i + x_right - x_i * x_right;
            }
        },
    );

    // Extract final state
    let last = num_steps - 1;
    let final_state: Vec<BaseElement> = (0..n).map(|i| trace.get(i, last)).collect();

    (trace, final_state)
}

/// Compute Rule 30 in plain u8 for quick validation.
pub fn rule30_plain(init: &[u8], steps: usize) -> Vec<u8> {
    let n = init.len();
    let mut state = init.to_vec();
    for _ in 0..steps {
        let mut next = vec![0u8; n];
        for i in 0..n {
            let left = state[(i + n - 1) % n];
            let center = state[i];
            let right = state[(i + 1) % n];
            // Rule 30: left XOR (center OR right)
            next[i] = left ^ (center | right);
        }
        state = next;
    }
    state
}
