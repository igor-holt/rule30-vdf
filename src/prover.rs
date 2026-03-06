// =============================================================================
// Prover + Verifier wrappers for Rule 30 VDF.
// Winterfell 0.13.1 API.
// =============================================================================

use winterfell::{
    crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree},
    math::{fields::f128::BaseElement, FieldElement},
    AcceptableOptions, BatchingMethod, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, FieldExtension, PartitionOptions,
    ProofOptions, Prover, StarkDomain, StarkProof, Trace, TraceInfo, TracePolyTable,
    TraceTable,
};
use winterfell::matrix::ColMatrix;
use winterfell::AuxRandElements;

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

    pub fn prove_trace(
        &self,
        trace: TraceTable<BaseElement>,
    ) -> Result<StarkProof, String> {
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

pub fn verify_proof(
    proof: StarkProof,
    pub_inputs: Rule30PublicInputs,
) -> Result<(), String> {
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
    ProofOptions::new(
        32,                        // num_queries
        8,                         // blowup_factor
        0,                         // grinding_factor
        FieldExtension::None,      // field_extension
        8,                         // FRI folding factor
        31,                        // FRI max remainder poly degree
        BatchingMethod::Linear,    // composition polynomial batching
        BatchingMethod::Linear,    // DEEP polynomial batching
    )
}
