use crate::base::encode::U256;
use curve25519_dalek::scalar::Scalar;

/// A trait for enabling zig-zag encoding
///
/// See https://developers.google.com/protocol-buffers/docs/encoding#signed-ints
/// for a descriptive reference.
pub trait ZigZag<T> {
    /// Encodes this ZigZag-enabled type into the type specified by implementation
    fn zigzag(&self) -> T;
}

/// Zigzag convertion from a dalek Scalar to a ZigZag u256 integer
///
/// For this conversion, we compute:
///
/// ```text
/// let x = *self;
/// let y = -self; // x + y = 0 ==> y = -x
/// ```
///
/// Then we choose the smallest value between `x` and `y`. Finally,
/// if `x` is the smallest value, we remap it to `2 * x` u256 integer,
/// which represents a positive ZigZag encoding.
/// Otherwise, we remap `y` to `2 * y + 1` u256 integer,
/// which represents a negative ZigZag encoding (-y).
impl ZigZag<U256> for Scalar {
    fn zigzag(&self) -> U256 {
        // since self is a dalek scalar, we never have the last bit 255 set
        // therefore, we should never expect overflow when multiplying by 2
        let mut x: U256 = self.into();
        let mut y: U256 = (&-self).into(); // x + y = 0 ==> y = -x

        // we return the smallest ZigZag number between x and y
        // in case x is bigger than y, we return -y (encoded in the ZigZag format)
        // otherwise, we simply return x (also in the ZigZag format).
        // doing that is a better memory-efficient approach, given that we can always
        // recover the value x from -y. After all, by construction we have `x + y = 0`.
        if x.high > y.high || (x.high == y.high && x.low > y.low) {
            // y is smaller than x
            // we multiply y by 2 and sum 1 (effectively encoding a ZigZag -y)
            y.high = (y.high << 1) | (y.low >> 127);
            y.low = (y.low << 1) | 1;

            y
        } else {
            // x is smaller than y
            // we multiply x by 2 (effectively encoding a ZigZag x)
            x.high = (x.high << 1) | (x.low >> 127);
            x.low <<= 1;

            x
        }
    }
}

/// Zigzag convertion from an u256 integer to a dalek Scalar.
///
/// For this conversion, we first verify if `self` is an odd or even number.
/// In case `self` is odd, the encoded number represents a negative
/// ZigZag value `-y`, encoded as `2 * y + 1`.
/// Otherwise, in case it's even, the encoded number represents
/// a positive ZigZag value `x`, encoded as `2 * x`.
///
/// In both cases, we divide the `self` value by 2 in order
/// to remove the ZigZag encoding (`y = self / 2` or `x = self / 2`).
///
/// Finally, we return either -1 * dalek::Scalar(y) or dalek::Scalar(x),
/// which in both cases represents the `x` scalar.
impl ZigZag<Scalar> for U256 {
    fn zigzag(&self) -> Scalar {
        // we need to divide self by 2 to remove the ZigZag encoding
        let zig_val = U256 {
            low: (self.low >> 1) | ((self.high & 1) << 127),
            high: self.high >> 1,
        };
        let scal: Scalar = (&zig_val).into();

        // verify if self is an odd or even number
        // in case it's an odd number, then scal represents the number `y`
        // otherwise, it represents the number x
        if self.low & 1 == 1 {
            // even though the encoding represented a -y,
            // scal actually represents a `y` (we simply divided self by 2).
            // Also, since x + y = 0, we need to compute -scal to return x
            -scal
        } else {
            // return x
            scal
        }
    }
}