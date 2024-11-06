use crate::base::{
    database::{Column, ColumnType, LiteralValue},
    scalar::Scalar,
};
use bumpalo::Bump;
use snafu::Snafu;

/// The result of evaluating an expression.
///
/// Inspired by [`datafusion_expr_common::ColumnarValue`]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ColumnarValue<'a, S: Scalar> {
    /// A [ `ColumnarValue::Column` ] is a list of values.
    Column(Column<'a, S>),
    /// A [ `ColumnarValue::Literal` ] is a single value with indeterminate size.
    Literal(LiteralValue<S>),
}

/// Errors from operations on [`ColumnarValue`]s.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ColumnarValueError {
    /// Attempt to convert a `[ColumnarValue::Column]` to a column of a different length
    ColumnLengthMismatch {
        /// The length of the `[ColumnarValue::Column]`
        columnar_value_length: usize,
        /// The length we attempted to convert the `[ColumnarValue::Column]` to
        attempt_to_convert_length: usize,
    },
}

impl<'a, S: Scalar> ColumnarValue<'a, S> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Column(column) => column.column_type(),
            Self::Literal(literal) => literal.column_type(),
        }
    }

    /// Converts the [`ColumnarValue`] to a [`Column`]
    pub fn into_column(
        &self,
        num_rows: usize,
        alloc: &'a Bump,
    ) -> Result<Column<'a, S>, ColumnarValueError> {
        match self {
            Self::Column(column) => {
                if column.len() == num_rows {
                    Ok(*column)
                } else {
                    Err(ColumnarValueError::ColumnLengthMismatch {
                        columnar_value_length: column.len(),
                        attempt_to_convert_length: num_rows,
                    })
                }
            }
            Self::Literal(literal) => {
                Ok(Column::from_literal_with_length(literal, num_rows, alloc))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use core::convert::Into;

    #[test]
    fn we_can_get_column_type_of_columnar_values() {
        let column = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        assert_eq!(column.column_type(), ColumnType::Int);

        let column = ColumnarValue::Literal(LiteralValue::<TestScalar>::Boolean(true));
        assert_eq!(column.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_can_transform_columnar_values_into_columns() {
        let bump = Bump::new();

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        let column = columnar_value.into_column(3, &bump).unwrap();
        assert_eq!(column, Column::Int(&[1, 2, 3]));

        let columnar_value = ColumnarValue::Literal(LiteralValue::<TestScalar>::Boolean(false));
        let column = columnar_value.into_column(5, &bump).unwrap();
        assert_eq!(column, Column::Boolean(&[false; 5]));

        // Check whether it works if `num_rows` is 0
        let columnar_value = ColumnarValue::Literal(LiteralValue::<TestScalar>::TinyInt(2));
        let column = columnar_value.into_column(0, &bump).unwrap();
        assert_eq!(column, Column::TinyInt(&[]));

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::SmallInt(&[]));
        let column = columnar_value.into_column(0, &bump).unwrap();
        assert_eq!(column, Column::SmallInt(&[]));
    }

    #[test]
    fn we_cannot_transform_columnar_values_into_columns_of_different_length() {
        let bump = Bump::new();

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        let res = columnar_value.into_column(2, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 3,
                attempt_to_convert_length: 2,
            })
        );

        let strings = ["a", "b", "c"];
        let scalars: Vec<TestScalar> = strings.iter().map(Into::into).collect();
        let columnar_value =
            ColumnarValue::Column(Column::<TestScalar>::VarChar((&strings, &scalars)));
        let res = columnar_value.into_column(0, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 3,
                attempt_to_convert_length: 0,
            })
        );

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Boolean(&[]));
        let res = columnar_value.into_column(1, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 0,
                attempt_to_convert_length: 1,
            })
        );
    }
}
