//! Module with database related functionality. In particular, this module contains the
//! accessor traits and the `OwnedTable` type along with some utility functions to convert
//! between Arrow and `OwnedTable`.
mod accessor;
pub use accessor::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor};

mod column;
pub use column::{Column, ColumnField, ColumnRef, ColumnType};

mod column_operation;
pub use column_operation::{
    try_add_subtract_column_types, try_divide_column_types, try_multiply_column_types,
};

mod column_operation_error;
pub use column_operation_error::{ColumnOperationError, ColumnOperationResult};

mod literal_value;
pub use literal_value::LiteralValue;

mod table_ref;
pub use table_ref::TableRef;

mod owned_column;
pub(crate) use owned_column::compare_indexes_by_owned_columns_with_direction;
pub use owned_column::OwnedColumn;

mod owned_column_error;
pub use owned_column_error::{OwnedColumnError, OwnedColumnResult};

/// TODO: add docs
pub(crate) mod owned_column_operation;

mod owned_table;
pub use owned_table::OwnedTable;
pub(crate) use owned_table::OwnedTableError;
#[cfg(test)]
mod owned_table_test;
pub mod owned_table_utility;

/// TODO: add docs
pub(crate) mod expression_evaluation;
mod expression_evaluation_error;
#[cfg(test)]
mod expression_evaluation_test;
pub use expression_evaluation_error::{ExpressionEvaluationError, ExpressionEvaluationResult};

#[cfg(any(test, feature = "test"))]
mod test_accessor;
#[cfg(any(test, feature = "test"))]
pub use test_accessor::TestAccessor;
#[cfg(test)]
pub(crate) use test_accessor::UnimplementedTestAccessor;

#[cfg(test)]
mod test_schema_accessor;
#[cfg(test)]
pub(crate) use test_schema_accessor::TestSchemaAccessor;

#[cfg(any(test, feature = "test"))]
mod owned_table_test_accessor;
#[cfg(any(test, feature = "test"))]
pub use owned_table_test_accessor::OwnedTableTestAccessor;
#[cfg(all(test, feature = "blitzar"))]
mod owned_table_test_accessor_test;

/// TODO: add docs
pub(crate) mod filter_util;
#[cfg(test)]
mod filter_util_test;

pub(crate) mod group_by_util;
#[cfg(test)]
mod group_by_util_test;
