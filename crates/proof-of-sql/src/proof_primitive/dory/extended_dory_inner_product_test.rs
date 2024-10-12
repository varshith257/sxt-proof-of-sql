use super::{
    extended_dory_inner_product_prove, extended_dory_inner_product_verify,
    extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs, rand_F_tensors,
    rand_G_vecs, test_rng, DoryMessages, ExtendedProverState, G1Affine, PublicParameters, GT,
};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn we_can_prove_and_verify_an_extended_dory_inner_product() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_can_prove_and_verify_an_extended_dory_inner_product_for_multiple_nu_values() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::test_rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();

    for nu in 0..max_nu {
        let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
        let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
        let mut messages = DoryMessages::default();
        extended_dory_inner_product_prove(
            &mut messages,
            &mut transcript,
            prover_state,
            &prover_setup,
        );

        let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
        assert!(extended_dory_inner_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup,
            extended_dory_reduce_verify_fold_s_vecs
        ));
    }
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_a_message_is_modified() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_there_are_too_few_GT_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages.pop();

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_there_are_too_many_GT_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages.push(GT::rand(&mut rng));

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_there_are_too_few_G1_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.G1_messages.pop();

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_there_are_too_many_G1_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.G1_messages.push(G1Affine::rand(&mut rng));

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_the_transcripts_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test_wrong");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_the_setups_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let pp_wrong = PublicParameters::test_rand(nu, &mut rng);
    let verifier_setup = (&pp_wrong).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_the_base_commitment_is_wrong() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    verifier_state.base_state.C = GT::rand(&mut rng).into();

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}

#[test]
fn we_fail_to_verify_an_extended_dory_inner_product_when_a_scalar_commitment_is_wrong() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    verifier_state.E_1 = G1Affine::rand(&mut rng).into();

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    let mut messages = DoryMessages::default();
    extended_dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"extended_dory_inner_product_test");
    assert!(!extended_dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs
    ));
}
