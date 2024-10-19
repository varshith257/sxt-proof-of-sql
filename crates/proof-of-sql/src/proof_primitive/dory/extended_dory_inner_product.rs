use super::{
    scalar_product_prove, scalar_product_verify, DoryMessages, ExtendedProverState,
    ExtendedVerifierState, ProverSetup, VerifierSetup, F,
};
use crate::{
    base::proof::Transcript,
    proof_primitive::dory::{
        extended_dory_reduce_prove, extended_dory_reduce_verify, fold_scalars_0_prove,
        fold_scalars_0_verify,
    },
};

/// This is the prover side of the extended Dory-Innerproduct algorithm in section 4.3 of https://eprint.iacr.org/2020/1274.pdf.
/// This function builds/enqueues `messages`, appends to `transcript`, and consumes `state`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_inner_product_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedProverState,
    setup: &ProverSetup,
) {
    let nu = state.base_state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        extended_dory_reduce_prove(messages, transcript, &mut state, setup);
    }
    let base_state = fold_scalars_0_prove(messages, transcript, state, setup);
    scalar_product_prove(messages, transcript, &base_state);
}

/// This is the verifier side of the extended Dory-Innerproduct algorithm in section 4.3 of https://eprint.iacr.org/2020/1274.pdf.
/// This function consumes/dequeues from `messages`, appends to `transcript`, and consumes `state`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_inner_product_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedVerifierState,
    setup: &VerifierSetup,
    fold_s_tensors_verify: impl Fn(&ExtendedVerifierState) -> (F, F),
) -> bool {
    let nu = state.base_state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        if !extended_dory_reduce_verify(messages, transcript, &mut state, setup) {
            return false;
        }
    }
    let base_state =
        fold_scalars_0_verify(messages, transcript, state, setup, fold_s_tensors_verify);
    scalar_product_verify(messages, transcript, base_state, setup)
}
