use super::arrow_array_to_column_conversion::ArrowArrayToColumnConversionError;
use crate::base::commitment::ColumnCommitmentsMismatch;
use proof_of_sql_parser::ParseError;
use snafu::Snafu;

#[cfg(feature = "arrow")]
#[derive(Debug, Snafu)]
pub enum RecordBatchToColumnsError {
    /// Error converting from arrow array
    #[snafu(transparent)]
    ArrowArrayToColumnConversionError {
        /// The underlying source error
        source: ArrowArrayToColumnConversionError,
    },
    #[snafu(transparent)]
    /// This error occurs when converting from a record batch name to an identifier fails.
    FieldParseFail {
        /// The underlying source error
        source: ParseError,
    },
}

#[cfg(feature = "arrow")]
#[derive(Debug, Snafu)]
pub enum AppendRecordBatchTableCommitmentError {
    #[snafu(transparent)]
    ColumnCommitmentsMismatch { source: ColumnCommitmentsMismatch },
    #[snafu(transparent)]
    ArrowBatchToColumnError { source: RecordBatchToColumnsError },
}
