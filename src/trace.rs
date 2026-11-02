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
pub fn build_trace(init: &[u8], num_steps: usize) -> (TraceTable<BaseElement>, Vec<BaseElement>) {
    let n = init.len();
    assert!(n > 0, "Rule 30 ring must contain at least one cell");
    // The last column is the row index. Constant Rule 30 traces (the all-zero
    // ring, the two-cell fixed point) otherwise make Winterfell's DEEP
    // composition polynomial drop below degree T-2, and the prover panics.
    let trace_width = 2 * n + 1;
    let mut trace = TraceTable::new(trace_width, num_steps);

    let one = BaseElement::ONE;
    let zero = BaseElement::ZERO;
    let two = BaseElement::from(2u32);

    let to_fe = |b: u8| -> BaseElement {
        if b != 0 {
            one
        } else {
            zero
        }
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
            state[2 * n] = zero;
        },
        // Update closure: called for rows 1..num_steps-1.
        // `state` contains the PREVIOUS row; we modify it in-place to become the CURRENT row.
        // `prev_step` is the index of that previous row.
        |prev_step, state: &mut [BaseElement]| {
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
            state[2 * n] = BaseElement::from((prev_step + 1) as u64);
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

#[cfg(test)]
mod tests {
    use super::{build_trace, rule30_plain};
    use winterfell::math::fields::f128::BaseElement;
    use winterfell::math::FieldElement;

    fn bits_of(cells: &[BaseElement]) -> Vec<u8> {
        cells
            .iter()
            .map(|cell| {
                if *cell == BaseElement::ZERO {
                    0
                } else if *cell == BaseElement::ONE {
                    1
                } else {
                    panic!("trace cell is not boolean: {cell:?}");
                }
            })
            .collect()
    }

    fn assert_trace_matches_plain(init: &[u8], steps: usize) {
        let (trace, final_state) = build_trace(init, steps);
        let n = init.len();
        for step in 0..steps {
            let row: Vec<BaseElement> = (0..n).map(|col| trace.get(col, step)).collect();
            assert_eq!(
                bits_of(&row),
                rule30_plain(init, step),
                "row {step} diverged from the bitwise simulation"
            );
            for i in 0..n {
                let x = trace.get(i, step);
                let x_right = trace.get((i + 1) % n, step);
                let w = trace.get(n + i, step);
                assert_eq!(
                    w,
                    x + x_right - x * x_right,
                    "OR witness at row {step} cell {i}"
                );
                if step + 1 < steps {
                    let x_left = trace.get((i + n - 1) % n, step);
                    let y = trace.get(i, step + 1);
                    let two = BaseElement::from(2u32);
                    assert_eq!(y, x_left + w - two * x_left * w);
                }
            }
            assert_eq!(
                trace.get(2 * n, step),
                BaseElement::from(step as u64),
                "row index column"
            );
        }
        assert_eq!(bits_of(&final_state), rule30_plain(init, steps - 1));
    }

    #[test]
    fn hand_computed_period_three_ring() {
        // One step on [1, 0, 0]: each cell's left/center/right neighborhood
        // yields [1, 1, 1], and the all-ones ring steps to all zeros.
        let init = [1u8, 0, 0];
        assert_eq!(rule30_plain(&init, 0), vec![1, 0, 0]);
        assert_eq!(rule30_plain(&init, 1), vec![1, 1, 1]);
        assert_eq!(rule30_plain(&init, 2), vec![0, 0, 0]);
        assert_eq!(rule30_plain(&init, 5), vec![0, 0, 0]);
    }

    #[test]
    fn fixed_points_and_single_cell() {
        assert_eq!(rule30_plain(&[0, 0, 0, 0], 17), vec![0, 0, 0, 0]);
        assert_eq!(rule30_plain(&[1, 1, 1, 1], 1), vec![0, 0, 0, 0]);
        // N = 2, [1, 0] is a fixed point of the periodic rule.
        assert_eq!(rule30_plain(&[1, 0], 9), vec![1, 0]);
        // A single cell is its own left and right neighbor, so 1 steps to 0.
        assert_eq!(rule30_plain(&[1], 1), vec![0]);
        assert_eq!(rule30_plain(&[1], 4), vec![0]);
        assert_eq!(rule30_plain(&[0], 4), vec![0]);
    }

    #[test]
    fn trace_matches_plain_on_edges_and_a_wide_ring() {
        assert_trace_matches_plain(&[1, 0, 0, 0, 0, 0, 0, 0], 8);
        assert_trace_matches_plain(&[0, 0, 0, 0, 0, 0, 0, 1], 16);
        assert_trace_matches_plain(&[1, 0, 1, 0, 1, 0, 1, 0], 32);
        assert_trace_matches_plain(&[0, 0, 0, 0, 0, 0, 0, 0], 8);
        assert_trace_matches_plain(&[1, 1, 1, 1, 1, 1, 1, 1], 8);
        assert_trace_matches_plain(&[1], 8);
        assert_trace_matches_plain(&[1, 0], 8);
        assert_trace_matches_plain(&[1, 0, 0], 8);

        let mut wide = vec![0u8; 64];
        wide[0] = 1;
        wide[32] = 1;
        wide[63] = 1;
        assert_trace_matches_plain(&wide, 256);
    }

    #[test]
    #[should_panic]
    fn trace_length_must_be_a_power_of_two() {
        let _ = build_trace(&[1, 0, 0, 0], 12);
    }

    #[test]
    #[should_panic]
    fn trace_length_must_be_at_least_8() {
        let _ = build_trace(&[1, 0, 0, 0], 4);
    }

    #[test]
    #[should_panic]
    fn empty_ring_is_rejected() {
        let _ = build_trace(&[], 8);
    }
}
