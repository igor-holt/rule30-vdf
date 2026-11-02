// =============================================================================
// Prover + Verifier wrappers for Rule 30 VDF.
// Winterfell 0.13.1 API.
// =============================================================================

use winterfell::matrix::ColMatrix;
use winterfell::AuxRandElements;
use winterfell::{
    crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree},
    math::{fields::f128::BaseElement, FieldElement},
    AcceptableOptions, BatchingMethod, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, FieldExtension, PartitionOptions, Proof, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable,
};

use crate::air::{Rule30Air, Rule30PublicInputs};

// -- Prover -------------------------------------------------------------------

pub struct Rule30Prover {
    options: ProofOptions,
    pub_inputs: Rule30PublicInputs,
}

impl Rule30Prover {
    pub fn new(options: ProofOptions, pub_inputs: Rule30PublicInputs) -> Self {
        Self {
            options,
            pub_inputs,
        }
    }

    pub fn prove_trace(&self, trace: TraceTable<BaseElement>) -> Result<Proof, String> {
        Prover::prove(self, trace).map_err(|e| format!("proof generation failed: {e}"))
    }
}

impl Prover for Rule30Prover {
    type BaseField = BaseElement;
    type Air = Rule30Air;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3_256<BaseElement>;
    type VC = MerkleTree<Self::HashFn>;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> Rule30PublicInputs {
        self.pub_inputs.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

// -- Verification wrapper -----------------------------------------------------

pub fn verify_proof(proof: Proof, pub_inputs: Rule30PublicInputs) -> Result<(), String> {
    let acceptable = AcceptableOptions::MinConjecturedSecurity(96);
    winterfell::verify::<
        Rule30Air,
        Blake3_256<BaseElement>,
        DefaultRandomCoin<Blake3_256<BaseElement>>,
        MerkleTree<Blake3_256<BaseElement>>,
    >(proof, pub_inputs, &acceptable)
    .map_err(|e| format!("verification failed: {e}"))
}

// -- Default proof options (8-arg form, 0.13 API) -----------------------------

pub fn default_proof_options() -> ProofOptions {
    // Winterfell's conjectured-security estimator is
    // `min(field_bits, queries * log2(blowup) + grinding) - 1`, capped by hash
    // collision resistance. 32 queries at blowup 8 estimate 95 bits, and
    // `verify_proof` rejects anything under 96. 33 queries estimate 98 bits.
    ProofOptions::new(
        33,                     // num_queries
        8,                      // blowup_factor
        0,                      // grinding_factor
        FieldExtension::None,   // field_extension
        8,                      // FRI folding factor
        31,                     // FRI max remainder poly degree
        BatchingMethod::Linear, // composition polynomial batching
        BatchingMethod::Linear, // DEEP polynomial batching
    )
}

#[cfg(test)]
mod tests {
    use super::default_proof_options;

    /// Mirrors `ConjecturedSecurity::compute` in Winterfell 0.13 for this
    /// 128-bit field and BLAKE3-256 (128-bit collision resistance).
    #[test]
    fn default_options_clear_the_96_bit_verifier_floor() {
        let options = default_proof_options();
        let mut query_security =
            options.blowup_factor().ilog2() * u32::try_from(options.num_queries()).unwrap();
        if query_security >= 80 {
            query_security += options.grinding_factor();
        }
        let field_security = 128 * options.field_extension().degree();
        let bits = std::cmp::min(std::cmp::min(field_security, query_security) - 1, 128);
        assert!(
            bits >= 96,
            "conjectured security {bits} is below verify_proof's floor of 96"
        );
        assert_eq!(bits, 98);
    }
}
