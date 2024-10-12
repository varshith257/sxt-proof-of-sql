use crate::base::if_rayon;
#[cfg(feature = "rayon")]
use ark_ec::pairing::MillerLoopOutput;
use ark_ec::pairing::{Pairing, PairingOutput};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_impl simply because tracing doesn't work well with threading.
pub fn pairing<P: Pairing>(
    p: impl Into<P::G1Prepared>,
    q: impl Into<P::G2Prepared>,
) -> PairingOutput<P> {
    Pairing::pairing(p, q)
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing<P: Pairing>(
    a: impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
    b: impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
) -> PairingOutput<P> {
    multi_pairing_impl(a, b)
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_2_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_2<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (PairingOutput<P>, PairingOutput<P>) {
    multi_pairing_2_impl((a0, b0), (a1, b1))
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_4_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_4<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a2, b2): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a3, b3): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
) {
    multi_pairing_4_impl((a0, b0), (a1, b1), (a2, b2), (a3, b3))
}
/// # Panics
/// This function may panic if the final exponentiation fails due to invalid inputs, or if the multi-pairing operation encounters an error with the provided elements.
fn multi_pairing_impl<P: Pairing>(
    a: impl IntoIterator<Item = impl Into<P::G1Prepared> + Send>,
    b: impl IntoIterator<Item = impl Into<P::G2Prepared> + Send>,
) -> PairingOutput<P> {
    if_rayon!(
        {
            let a: Vec<_> = a.into_iter().collect();
            let b: Vec<_> = b.into_iter().collect();
            Pairing::final_exponentiation(MillerLoopOutput(
                a.into_par_iter()
                    .zip(b)
                    .map(|(x, y)| P::miller_loop(x, y).0)
                    .product(),
            ))
            .unwrap()
        },
        Pairing::multi_pairing(a, b)
    )
}
fn multi_pairing_2_impl<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (PairingOutput<P>, PairingOutput<P>) {
    if_rayon!(
        rayon::join(|| multi_pairing_impl(a0, b0), || multi_pairing_impl(a1, b1)),
        (multi_pairing_impl(a0, b0), multi_pairing_impl(a1, b1))
    )
}
fn multi_pairing_4_impl<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a2, b2): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a3, b3): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
) {
    let ((c0, c1), (c2, c3)) = if_rayon!(
        rayon::join(
            || multi_pairing_2_impl((a0, b0), (a1, b1)),
            || multi_pairing_2_impl((a2, b2), (a3, b3)),
        ),
        (
            multi_pairing_2_impl((a0, b0), (a1, b1)),
            multi_pairing_2_impl((a2, b2), (a3, b3)),
        )
    );
    (c0, c1, c2, c3)
}
