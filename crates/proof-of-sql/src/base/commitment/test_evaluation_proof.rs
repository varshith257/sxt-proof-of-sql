use super::{naive_commitment::NaiveCommitment, CommitmentEvaluationProof};
use crate::base::{proof::Transcript, scalar::test_scalar::TestScalar};

/// This should only be used for the purpose of unit testing.
pub struct TestEvaluationProof {}

/// This should only be used for the purpose of unit testing.
/// For now it is only being created for the purpose of implementing
/// [`CommitmentEvaluationProof`] for [`TestEvaluationProof`].
pub enum TestErrorType {}

impl CommitmentEvaluationProof for TestEvaluationProof {
    type Scalar = TestScalar;

    type Commitment = NaiveCommitment;

    type Error = TestErrorType;

    type ProverPublicSetup<'a> = ();

    type VerifierPublicSetup<'a> = ();

    fn new(
        _transcript: &mut impl Transcript,
        _a: &[Self::Scalar],
        _b_point: &[Self::Scalar],
        _generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        unimplemented!("The `CommitmentEvaluationProof` methods are unimplemented for `TestEvaluationProof`. There is nothing preventing a naive implementation here. If this gets done, this type should likely be renamed as `NaiveEvaluationProof` to reflect this.")
    }

    fn verify_batched_proof(
        &self,
        _transcript: &mut impl Transcript,
        _commit_batch: &[Self::Commitment],
        _batching_factors: &[Self::Scalar],
        _product: &Self::Scalar,
        _b_point: &[Self::Scalar],
        _generators_offset: u64,
        _table_length: usize,
        _setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
