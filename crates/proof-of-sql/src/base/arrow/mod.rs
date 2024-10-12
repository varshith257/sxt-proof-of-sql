//! This module provides conversions and utilities for working with Arrow data structures.
//! It includes conversions between Arrow arrays and columns, handling of record batches,
//! and utilities for scalar and i256 conversions.
#[cfg(feature = "arrow")]
pub mod arrow_array_to_column_conversion;
#[cfg(feature = "arrow")]
pub mod owned_and_arrow_conversions;
#[cfg(all(test, feature = "arrow"))]
mod owned_and_arrow_conversions_test;
#[cfg(feature = "arrow")]
pub mod record_batch_conversion;
#[cfg(feature = "arrow")]
pub mod record_batch_errors;
#[cfg(feature = "arrow")]
pub mod record_batch_utility;
#[cfg(feature = "arrow")]
pub mod scalar_and_i256_conversions;
