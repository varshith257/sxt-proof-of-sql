use super::{scale_and_add_subtract_eval, scale_and_subtract, DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable AST expression for an equals expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualsExpr<C: Commitment> {
    lhs: Box<DynProofExpr<C>>,
    rhs: Box<DynProofExpr<C>>,
}

impl<C: Commitment> EqualsExpr<C> {
    /// Create a new equals expression
    pub fn new(lhs: Box<DynProofExpr<C>>, rhs: Box<DynProofExpr<C>>) -> Self {
        Self { lhs, rhs }
    }
}

impl<C: Commitment> ProofExpr<C> for EqualsExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_equals_zero(builder);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "EqualsExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column = self.rhs.result_evaluate(table_length, alloc, accessor);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res = scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, true)
            .expect("Failed to scale and subtract");
        Column::Boolean(result_evaluate_equals_zero(table_length, alloc, res))
    }

    #[tracing::instrument(name = "EqualsExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, accessor);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res = scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, true)
            .expect("Failed to scale and subtract");
        Column::Boolean(prover_evaluate_equals_zero(builder, alloc, res))
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res = scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, true);
        Ok(verifier_evaluate_equals_zero(builder, res))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[allow(
    clippy::missing_panics_doc,
    reason = "table_length is guaranteed to match lhs.len()"
)]
pub fn result_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == S::zero())
}

pub fn prover_evaluate_equals_zero<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    let table_length = builder.table_length();

    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] = alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != S::zero());
    builder.produce_intermediate_mle(selection_not);

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(S::one(), vec![Box::new(lhs), Box::new(selection)])],
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(selection_not)]),
            (
                -S::one(),
                vec![Box::new(lhs), Box::new(lhs_pseudo_inv as &[_])],
            ),
        ],
    );

    selection
}

pub fn verifier_evaluate_equals_zero<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    lhs_eval: C::Scalar,
) -> C::Scalar {
    // consume mle evaluations
    let lhs_pseudo_inv_eval = builder.consume_intermediate_mle();
    let selection_not_eval = builder.consume_intermediate_mle();
    let selection_eval = builder.mle_evaluations.input_one_evaluation - selection_not_eval;

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::Identity,
        selection_eval * lhs_eval,
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::Identity,
        selection_not_eval - lhs_eval * lhs_pseudo_inv_eval,
    );

    selection_eval
}

pub fn count_equals_zero(builder: &mut CountBuilder) {
    builder.count_subpolynomials(2);
    builder.count_intermediate_mles(2);
    builder.count_degree(3);
}
