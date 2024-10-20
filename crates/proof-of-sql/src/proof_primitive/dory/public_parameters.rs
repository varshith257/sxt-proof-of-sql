use super::{G1Affine, G2Affine};
use alloc::vec::Vec;
use ark_ff::UniformRand;
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use ark_std::rand::{CryptoRng, Rng};
use core::iter;
#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Write},
    path::Path,
};

/// The public parameters for the Dory protocol. See section 5 of <https://eprint.iacr.org/2020/1274.pdf> for details.
///
/// Note: even though `H_1` and `H_2` are marked as blue, they are still needed.
///
/// Note: `Gamma_1_fin` is unused, so we leave it out.
pub struct PublicParameters {
    /// This is the vector of G1 elements that are used in the Dory protocol. That is, `Γ_1,0` in the Dory paper.
    pub(super) Gamma_1: Vec<G1Affine>,
    /// This is the vector of G2 elements that are used in the Dory protocol. That is, `Γ_2,0` in the Dory paper.
    pub(super) Gamma_2: Vec<G2Affine>,
    /// `H_1` = `H_1` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1Affine,
    /// `H_2` = `H_2` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2Affine,
    /// `Gamma_2_fin` = `Gamma_2,fin` in the Dory paper.
    pub(super) Gamma_2_fin: G2Affine,
    /// `max_nu` is the maximum nu that this setup will work for.
    pub(super) max_nu: usize,
}

impl PublicParameters {
    /// Generate cryptographically secure random public parameters.
    pub fn rand<R: CryptoRng + Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        Self::rand_impl(max_nu, rng)
    }
    #[cfg(any(test, feature = "test"))]
    /// Generate random public parameters for testing.
    pub fn test_rand<R: Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        Self::rand_impl(max_nu, rng)
    }
    fn rand_impl<R: Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        let (Gamma_1, Gamma_2) = iter::repeat_with(|| (G1Affine::rand(rng), G2Affine::rand(rng)))
            .take(1 << max_nu)
            .unzip();
        let (H_1, H_2) = (G1Affine::rand(rng), G2Affine::rand(rng));
        let Gamma_2_fin = G2Affine::rand(rng);

        Self {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
        }
    }
    #[cfg(feature = "std")]
    /// Function to save `PublicParameters` to a file in binary form
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Create or open the file at the specified path
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Serialize the PublicParameters struct into the file
        let mut serialized_data = Vec::new();
        self.serialize_with_mode(&mut serialized_data, Compress::No)
            .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))?;

        // Write serialized bytes to the file
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }
    #[cfg(feature = "std")]
    /// Function to load `PublicParameters` from a file in binary form
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        // Open the file at the specified path
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read the serialized data from the file
        let mut serialized_data = Vec::new();
        reader.read_to_end(&mut serialized_data)?;

        // Deserialize the data into a PublicParameters instance
        PublicParameters::deserialize_with_mode(
            &mut &serialized_data[..],
            Compress::No,
            Validate::Yes,
        )
        .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))
    }
}

impl CanonicalSerialize for PublicParameters {
    fn serialize_with_mode<W: ark_serialize::Write>(
        &self,
        mut writer: W,
        compress: ark_serialize::Compress,
    ) -> Result<(), SerializationError> {
        // Serialize max_nu (usize as u64)
        (self.max_nu as u64).serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_1 (Vec<G1Affine>)
        self.Gamma_1
            .iter()
            .try_for_each(|g1| g1.serialize_with_mode(&mut writer, compress))?;

        // Serialize Gamma_2 (Vec<G2Affine>)
        self.Gamma_2
            .iter()
            .try_for_each(|g2| g2.serialize_with_mode(&mut writer, compress))?;

        // Serialize H_1 (G1Affine)
        self.H_1.serialize_with_mode(&mut writer, compress)?;

        // Serialize H_2 (G2Affine)
        self.H_2.serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_2_fin (G2Affine)
        self.Gamma_2_fin
            .serialize_with_mode(&mut writer, compress)?;

        Ok(())
    }

    // Update serialized_size accordingly
    fn serialized_size(&self, compress: ark_serialize::Compress) -> usize {
        // Size of max_nu (u64 is 8 bytes)
        let max_nu_size = 8;

        // Size of Gamma_1 (Vec<G1Affine>)
        let gamma_1_size: usize = self
            .Gamma_1
            .iter()
            .map(|g1| g1.serialized_size(compress))
            .sum();

        // Size of Gamma_2 (Vec<G2Affine>)
        let gamma_2_size: usize = self
            .Gamma_2
            .iter()
            .map(|g2| g2.serialized_size(compress))
            .sum();

        // Size of H_1 (G1Affine)
        let h1_size = self.H_1.serialized_size(compress);

        // Size of H_2 (G2Affine)
        let h2_size = self.H_2.serialized_size(compress);

        // Size of Gamma_2_fin (G2Affine)
        let gamma_2_fin_size = self.Gamma_2_fin.serialized_size(compress);

        // Sum everything to get the total size
        max_nu_size + gamma_1_size + gamma_2_size + h1_size + h2_size + gamma_2_fin_size
    }
}

impl CanonicalDeserialize for PublicParameters {
    fn deserialize_with_mode<R: ark_serialize::Read>(
        mut reader: R,
        compress: ark_serialize::Compress,
        validate: ark_serialize::Validate,
    ) -> Result<Self, SerializationError> {
        // Deserialize max_nu (u64)
        let max_nu_u64 = u64::deserialize_with_mode(&mut reader, compress, validate)?;
        let max_nu: usize = max_nu_u64
            .try_into()
            .map_err(|_| SerializationError::InvalidData)?;

        // Deserialize Gamma_1 (Vec<G1Affine>)
        let Gamma_1: Vec<G1Affine> = (0..(1 << max_nu))
            .map(|_| G1Affine::deserialize_with_mode(&mut reader, compress, validate))
            .collect::<Result<_, _>>()?;

        // Deserialize Gamma_2 (Vec<G2Affine>)
        let Gamma_2: Vec<G2Affine> = (0..(1 << max_nu))
            .map(|_| G2Affine::deserialize_with_mode(&mut reader, compress, validate))
            .collect::<Result<_, _>>()?;

        // Deserialize H_1 (G1Affine)
        let H_1 = G1Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize H_2 (G2Affine)
        let H_2 = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize Gamma_2_fin (G2Affine)
        let Gamma_2_fin = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        Ok(Self {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
        })
    }

    // Remove unnecessary methods if they're not overridden
}

// Implement the Valid trait to perform validation on deserialized data
impl Valid for PublicParameters {
    fn check(&self) -> Result<(), SerializationError> {
        // Check that all G1Affine and G2Affine elements are valid
        self.Gamma_1
            .iter()
            .try_for_each(ark_serialize::Valid::check)?;
        self.Gamma_2
            .iter()
            .try_for_each(ark_serialize::Valid::check)?;

        self.H_1.check()?;
        self.H_2.check()?;
        self.Gamma_2_fin.check()?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_std::rand::thread_rng;
    use std::io::Cursor;

    #[test]
    fn we_can_serialize_and_deserialize_round_trip() {
        // Create a random PublicParameters instance
        let mut rng = thread_rng();
        let original_params = PublicParameters::rand(2, &mut rng);

        // Serialize the original parameters to a byte buffer
        let mut serialized_data = Vec::new();
        original_params
            .serialize_with_mode(&mut serialized_data, ark_serialize::Compress::No)
            .expect("Failed to serialize PublicParameters");

        // Deserialize the byte buffer back into a PublicParameters instance
        let mut reader = Cursor::new(serialized_data);
        let deserialized_params = PublicParameters::deserialize_with_mode(
            &mut reader,
            ark_serialize::Compress::No,
            ark_serialize::Validate::Yes,
        )
        .expect("Failed to deserialize PublicParameters");

        // Check that the original and deserialized parameters are the same
        assert_eq!(original_params.Gamma_1, deserialized_params.Gamma_1);
        assert_eq!(original_params.Gamma_2, deserialized_params.Gamma_2);
        assert_eq!(original_params.H_1, deserialized_params.H_1);
        assert_eq!(original_params.H_2, deserialized_params.H_2);
        assert_eq!(original_params.Gamma_2_fin, deserialized_params.Gamma_2_fin);
        assert_eq!(original_params.max_nu, deserialized_params.max_nu);

        // Validate the deserialized parameters to ensure correctness
        deserialized_params
            .check()
            .expect("Deserialized parameters are not valid");
    }

    // 13th Gen Intel® Core™ i9-13900H × 20
    // nu vs proof size & time:
    // nu = 4  |  0.005 MB  | 287.972567ms
    // nu = 10 |  0.282 MB  | 16.130250627s
    // nu = 12 |  1.125 MB  | 64.036526973s
    // nu = 14 |  4.500 MB  | 254.316791697s
    // nu = 15 |  9.000 MB  | 504.351756724s
    #[test]
    fn we_can_read_and_write_a_file_round_trip() {
        let nu_values = vec![1, 2, 4];

        // Loop through each nu value
        for &nu in &nu_values {
            println!("\nTesting with nu = {nu}");

            let start_time = std::time::Instant::now();

            // Create a random PublicParameters instance with the current nu value
            let mut rng = thread_rng();
            let original_params = PublicParameters::rand(nu, &mut rng);

            // File path in the current working directory
            let file_name = format!("public_params_{nu}.bin");
            let file_path = Path::new(&file_name);

            original_params
                .save_to_file(file_path)
                .expect("Failed to save PublicParameters to file");

            // Load the PublicParameters from the file
            let loaded_params = PublicParameters::load_from_file(file_path)
                .expect("Failed to load PublicParameters from file");

            // Check that the original and loaded parameters are identical
            assert_eq!(original_params.Gamma_1, loaded_params.Gamma_1);
            assert_eq!(original_params.Gamma_2, loaded_params.Gamma_2);
            assert_eq!(original_params.H_1, loaded_params.H_1);
            assert_eq!(original_params.H_2, loaded_params.H_2);
            assert_eq!(original_params.Gamma_2_fin, loaded_params.Gamma_2_fin);
            assert_eq!(original_params.max_nu, loaded_params.max_nu);

            // Record the file size in bytes
            let metadata = std::fs::metadata(file_path).expect("Failed to get file metadata");
            let file_size = metadata.len(); // Get the file size in bytes
            println!("File size for nu = {nu}: {file_size} bytes");

            // Record the time taken and print it
            let elapsed_time = start_time.elapsed();
            println!("Time taken for nu = {nu}: {elapsed_time:?}");

            // Clean up the test file after the test runs
            std::fs::remove_file(file_path).expect("Failed to remove test file");
        }
    }
}
