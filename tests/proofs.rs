//! STARK proof tests for the Rule 30 VDF.
//!
//! Trace checks that do not build a proof live in `src/trace.rs`. These tests
//! run the real prover and verifier, so they are slower.

use winterfell::crypto::hashers::Blake3_256;
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;
use winterfell::Proof;

use rule30_vdf::air::Rule30PublicInputs;
use rule30_vdf::prover::{default_proof_options, verify_proof, Rule30Prover};
use rule30_vdf::trace::{build_trace, rule30_plain};

fn to_field(bits: &[u8]) -> Vec<BaseElement> {
    bits.iter()
        .map(|&bit| {
            assert!(bit <= 1, "tests only use boolean cells");
            if bit == 0 {
                BaseElement::ZERO
            } else {
                BaseElement::ONE
            }
        })
        .collect()
}

fn to_bits(cells: &[BaseElement]) -> Vec<u8> {
    cells
        .iter()
        .map(|cell| u8::from(*cell == BaseElement::ONE))
        .collect()
}

fn center_seed(num_cells: usize) -> Vec<u8> {
    let mut init = vec![0u8; num_cells];
    init[num_cells / 2] = 1;
    init
}

fn prove(init: &[u8], steps: usize) -> (Proof, Rule30PublicInputs) {
    let (trace, final_state) = build_trace(init, steps);
    let expected = rule30_plain(init, steps - 1);
    assert_eq!(
        to_bits(&final_state),
        expected,
        "prover trace diverged from the naive simulation"
    );

    let pub_inputs = Rule30PublicInputs {
        init_state: to_field(init),
        final_state,
        num_cells: init.len(),
    };
    let prover = Rule30Prover::new(default_proof_options(), pub_inputs.clone());
    let proof = prover.prove_trace(trace).expect("proof generation failed");
    (proof, pub_inputs)
}

fn assert_verifies(proof: &Proof, pub_inputs: &Rule30PublicInputs) {
    let bits = proof
        .conjectured_security::<Blake3_256<BaseElement>>()
        .bits();
    assert!(
        bits >= 96,
        "Winterfell conjectured security was {bits} bits; verify_proof requires 96"
    );
    verify_proof(proof.clone(), pub_inputs.clone()).expect("honest proof was rejected");
}

#[test]
fn proofs_verify_for_several_delay_lengths() {
    // Trace length is the delay parameter. Row 0 is the input, so the proved
    // computation is `steps - 1` Rule 30 transitions.
    for steps in [8_usize, 32, 128] {
        let init = center_seed(8);
        let (proof, pub_inputs) = prove(&init, steps);
        assert_eq!(
            to_bits(&pub_inputs.final_state),
            rule30_plain(&init, steps - 1)
        );
        assert_verifies(&proof, &pub_inputs);
    }
}

#[test]
fn production_width_short_delay_verifies() {
    let init = center_seed(64);
    let (proof, pub_inputs) = prove(&init, 16);
    assert_verifies(&proof, &pub_inputs);
}

#[test]
fn edge_initial_states_verify() {
    // The all-zero ring and the N=2 pattern [1, 0] are fixed points. Their
    // Rule 30 columns are constant, which used to panic inside Winterfell.
    let cases: &[Vec<u8>] = &[
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1],
        vec![1, 0],
        vec![1, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1],
    ];
    for init in cases {
        let (proof, pub_inputs) = prove(init, 16);
        assert_verifies(&proof, &pub_inputs);
    }
}

#[test]
fn wrong_output_is_rejected() {
    let init = center_seed(8);
    let (proof, pub_inputs) = prove(&init, 16);
    assert_verifies(&proof, &pub_inputs);

    let mut wrong = pub_inputs.clone();
    let first = wrong.final_state[0];
    wrong.final_state[0] = if first == BaseElement::ZERO {
        BaseElement::ONE
    } else {
        BaseElement::ZERO
    };
    let err = verify_proof(proof.clone(), wrong).expect_err("flipped output was accepted");
    assert!(
        err.contains("verification failed"),
        "unexpected rejection: {err}"
    );

    let mut wrong_init = pub_inputs.clone();
    wrong_init.init_state[1] = BaseElement::ONE;
    let err = verify_proof(proof, wrong_init).expect_err("flipped input was accepted");
    assert!(
        err.contains("verification failed"),
        "unexpected rejection: {err}"
    );
}

#[test]
fn tampered_proof_is_rejected() {
    let init = center_seed(8);
    let (proof, pub_inputs) = prove(&init, 16);
    let bytes = proof.to_bytes();
    assert!(bytes.len() > 8, "proof was unexpectedly tiny");

    let mut positions = vec![0, bytes.len() / 3, bytes.len() / 2, bytes.len() - 1];
    positions.sort_unstable();
    positions.dedup();

    for index in positions {
        let mut tampered = bytes.clone();
        tampered[index] ^= 0xff;
        match Proof::from_bytes(&tampered) {
            Err(_) => {}
            Ok(parsed) => {
                let err = verify_proof(parsed, pub_inputs.clone())
                    .expect_err("tampered proof at byte {index} was accepted");
                assert!(
                    err.contains("verification failed"),
                    "unexpected rejection at byte {index}: {err}"
                );
            }
        }
    }

    assert!(
        Proof::from_bytes(&[0_u8; 8]).is_err(),
        "a truncated all-zero blob deserialized as a proof"
    );
}
