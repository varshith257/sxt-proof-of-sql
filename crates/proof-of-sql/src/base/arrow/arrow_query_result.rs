use crate::base::scalar::Scalar;
use crate::sql::proof::QueryData;
use arrow::{error::ArrowError, record_batch::RecordBatch};

#[cfg(feature = "arrow")]
impl<S: Scalar> TryFrom<QueryData<S>> for RecordBatch {
    type Error = ArrowError;

    fn try_from(value: QueryData<S>) -> Result<Self, Self::Error> {
        Self::try_from(value.table)
    }
}
