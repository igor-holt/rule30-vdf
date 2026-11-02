// =============================================================================
// AIR definition for Rule 30 VDF over F_p (128-bit prime field)
// Implements AIR-12 through AIR-16 constraints.
// =============================================================================

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, ToElements},
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};

// -- Public inputs: initial state + claimed final state ----------------------

#[derive(Clone, Debug)]
pub struct Rule30PublicInputs {
    pub init_state: Vec<BaseElement>,
    pub final_state: Vec<BaseElement>,
    pub num_cells: usize,
}

impl ToElements<BaseElement> for Rule30PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut elems = self.init_state.clone();
        elems.extend_from_slice(&self.final_state);
        elems
    }
}

// -- AIR struct ---------------------------------------------------------------

pub struct Rule30Air {
    context: AirContext<BaseElement>,
    init_state: Vec<BaseElement>,
    final_state: Vec<BaseElement>,
    num_cells: usize,
}

impl Air for Rule30Air {
    type BaseField = BaseElement;
    type PublicInputs = Rule30PublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let n = pub_inputs.num_cells;

        // 4*N degree-2 Rule 30 constraints, plus one degree-1 row-index constraint.
        //   N  boolean on x_i
        //   N  boolean on w_i
        //   N  OR gate
        //   N  XOR transition
        //   1  t' = t + 1
        let mut degrees = vec![TransitionConstraintDegree::new(2); 4 * n];
        degrees.push(TransitionConstraintDegree::new(1));

        // 2*N endpoint assertions on x, plus t_0 = 0.
        let context = AirContext::new(trace_info, degrees, 2 * n + 1, options);

        Self {
            context,
            init_state: pub_inputs.init_state,
            final_state: pub_inputs.final_state,
            num_cells: n,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Evaluate the Rule 30 transition constraints and the row-index constraint.
    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        let n = self.num_cells;
        let two = E::from(2u32);

        for i in 0..n {
            let x_i = current[i];
            let x_right = current[(i + 1) % n];
            let x_left = current[(i + n - 1) % n];
            let w_i = current[n + i];
            let y_i = next[i]; // next-row cell value

            // AIR-12: P_bool(x_i) = x_i^2 - x_i
            result[i] = x_i * x_i - x_i;

            // AIR-12: P_bool(w_i) = w_i^2 - w_i
            result[n + i] = w_i * w_i - w_i;

            // AIR-14: P_or = w_i - (x_i + x_right - x_i * x_right)
            result[2 * n + i] = w_i - (x_i + x_right - x_i * x_right);

            // AIR-15: P_xor = y_i - (x_left + w_i - 2 * x_left * w_i)
            result[3 * n + i] = y_i - (x_left + w_i - two * x_left * w_i);
        }

        // Row index advances by one. This column is also what keeps the DEEP
        // composition polynomial at full degree when the Rule 30 state is constant.
        let row_index = 2 * n;
        result[4 * n] = next[row_index] - (current[row_index] + E::ONE);
    }

    /// Boundary assertions: pin initial and final states.
    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let n = self.num_cells;
        let last_step = self.trace_length() - 1;
        let mut assertions = Vec::with_capacity(2 * n + 1);

        for i in 0..n {
            // First row: x_i = init_state[i]
            assertions.push(Assertion::single(i, 0, self.init_state[i]));
            // Last row: x_i = final_state[i]
            assertions.push(Assertion::single(i, last_step, self.final_state[i]));
        }
        assertions.push(Assertion::single(2 * n, 0, Self::BaseField::ZERO));

        assertions
    }
}
