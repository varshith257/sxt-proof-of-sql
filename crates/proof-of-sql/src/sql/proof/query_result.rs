use crate::base::{
    database::{OwnedTable, OwnedTableError},
    proof::ProofError,
    scalar::Scalar,
};
use snafu::Snafu;

/// Verifiable query errors
#[derive(Snafu, Debug)]
pub enum QueryError {
    /// The query result overflowed. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a result that was too large.
    #[snafu(display("Overflow error"))]
    Overflow,
    /// The query result string could not be decoded. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a string that was not valid UTF-8.
    #[snafu(display("String decode error"))]
    InvalidString,
    /// Decoding errors other than overflow and invalid string.
    #[snafu(display("Miscellaneous decoding error"))]
    MiscellaneousDecodingError,
    /// Miscellaneous evaluation error.
    #[snafu(display("Miscellaneous evaluation error"))]
    MiscellaneousEvaluationError,
    /// The proof failed to verify.
    #[snafu(transparent)]
    ProofError {
        /// The underlying source error
        source: ProofError,
    },
    /// The table data was invalid. This should never happen because this should get caught by the verifier before reaching this point.
    #[snafu(transparent)]
    InvalidTable {
        /// The underlying source error
        source: OwnedTableError,
    },
    /// The number of columns in the table was invalid.
    #[snafu(display("Invalid number of columns"))]
    InvalidColumnCount,
}

/// The verified results of a query along with metadata produced by verification
pub struct QueryData<S: Scalar> {
    /// We use Apache Arrow's [`RecordBatch`] to represent a table
    /// result so as to allow for easy interoperability with
    /// Apache Arrow Flight.
    ///
    /// See `<https://voltrondata.com/blog/apache-arrow-flight-primer/>`
    pub table: OwnedTable<S>,
    /// Additionally, there is a 32-byte verification hash that is included with this table.
    /// This hash provides evidence that the verification has been run.
    pub verification_hash: [u8; 32],
}

/// The result of a query -- either an error or a table.
pub type QueryResult<S> = Result<QueryData<S>, QueryError>;
