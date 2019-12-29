
/// Any object implementing SpecificEndian<T> can be converted between big and little endian.  Implement this trait to allow for endian conversion by this crate.
pub trait SpecificEndian<T> where Self: Into<T> + Clone + Copy {
    fn to_big_endian(&self) -> T;
    fn to_little_endian(&self) -> T;
    fn from_big_endian(&self) -> T;
    fn from_little_endian(&self) -> T;

}

/// A macro implementing SpecificEndian<T> for simple data types where big and little endian forms are the same.
macro_rules! make_specific_endian_single_byte {
    ($wrap_ty:ty) => {

        impl SpecificEndian<$wrap_ty> for $wrap_ty {
            fn to_big_endian(&self) -> Self {
                *self
            }
            fn to_little_endian(&self) -> Self {
                *self
            }
            fn from_big_endian(&self) -> Self {
                *self
            }
            fn from_little_endian(&self) -> Self {
                *self
            }
        }
    }
}

make_specific_endian_single_byte!(u8);
make_specific_endian_single_byte!(i8);
// If bool ends up being represented by something other than a byte, this might not work right.
make_specific_endian_single_byte!(bool);

/// A macro for implementing SpecificEndian<T> on types that have endian conversions built into Rust.  Currently, this is the primitive integer types.
macro_rules! make_specific_endian_integer {
    ($wrap_ty:ty) => {

        impl SpecificEndian<$wrap_ty> for $wrap_ty {
            fn to_big_endian(&self) -> Self {
                self.to_be()
            }
            fn to_little_endian(&self) -> Self {
                self.to_le()
            }
            fn from_big_endian(&self) -> Self {
                Self::from_be(*self)
            }
            fn from_little_endian(&self) -> Self {
                Self::from_le(*self)
            }
        }
    }
}

make_specific_endian_integer!(u16);
make_specific_endian_integer!(i16);
make_specific_endian_integer!(u32);
make_specific_endian_integer!(i32);
make_specific_endian_integer!(u64);
make_specific_endian_integer!(i64);
make_specific_endian_integer!(u128);
make_specific_endian_integer!(i128);
make_specific_endian_integer!(usize);
make_specific_endian_integer!(isize);

/// Uses .from_bits() and .to_bits() to implement SpecificEndian<T> with Integer types.  Can be used with any type having these methods, but mainly for use with the floats.
macro_rules! make_specific_endian_float {
    ($wrap_ty:ty) => {

        impl SpecificEndian<$wrap_ty> for $wrap_ty {
            fn to_big_endian(&self) -> Self {
                Self::from_bits(self.to_bits().to_be())
            }
            fn to_little_endian(&self) -> Self {
                Self::from_bits(self.to_bits().to_le())
            }
            fn from_big_endian(&self) -> Self {
                Self::from_bits(self.to_bits().from_big_endian())
            }
            fn from_little_endian(&self) -> Self {
                Self::from_bits(self.to_bits().from_little_endian())
            }
        }
    }
}

make_specific_endian_float!(f32);
make_specific_endian_float!(f64);

/// A big-endian representation of type T that implements SpecificEndian<T>.  Data stored in the struct must be converted to big-endian using from() or into().
#[derive(Copy, Clone, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BigEndian<T: SpecificEndian<T>> {pub(crate) _v: T}
unsafe impl<T: Send + SpecificEndian<T>> Send for BigEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for BigEndian<T> {}


impl<T> BigEndian<T> where T: SpecificEndian<T> {
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a BigEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self{_v: v}
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_big_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for BigEndian<T> {
    fn from(v: T) -> BigEndian<T> {
        BigEndian::<T>{_v: v.to_big_endian()}
    }
}


/// A little-endian representation of type T that implements SpecificEndian<T>.  Data stored in the struct must be converted to little-endian using from() or into().
#[derive(Copy, Clone, Hash, Debug, Default)]
#[repr(transparent)]
pub struct LittleEndian<T: SpecificEndian<T>> {pub(crate) _v: T}
unsafe impl<T: Send + SpecificEndian<T>> Send for LittleEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for LittleEndian<T> {}

impl<T> LittleEndian<T> where T: SpecificEndian<T> {
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a LittleEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self{_v: v}
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_little_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for LittleEndian<T> {
    fn from(v: T) -> LittleEndian<T> {
        LittleEndian::<T>{_v: v.to_little_endian()}
    }
}

// Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
macro_rules! make_primitive_type_from_be {
    ($wrap_ty:ty) => {

        impl From<BigEndian<$wrap_ty>> for $wrap_ty {
            fn from(v: BigEndian<$wrap_ty>) -> $wrap_ty {
                v._v.from_big_endian()
            }
        }

    }
}

make_primitive_type_from_be!(bool);
make_primitive_type_from_be!(u8);
make_primitive_type_from_be!(i8);
make_primitive_type_from_be!(u16);
make_primitive_type_from_be!(i16);
make_primitive_type_from_be!(u32);
make_primitive_type_from_be!(i32);
make_primitive_type_from_be!(u64);
make_primitive_type_from_be!(i64);
make_primitive_type_from_be!(u128);
make_primitive_type_from_be!(i128);
make_primitive_type_from_be!(usize);
make_primitive_type_from_be!(isize);
make_primitive_type_from_be!(f32);
make_primitive_type_from_be!(f64);

// Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
macro_rules! make_primitive_type_from_le {
    ($wrap_ty:ty) => {

        impl From<LittleEndian<$wrap_ty>> for $wrap_ty {
            fn from(v: LittleEndian<$wrap_ty>) -> $wrap_ty {
                v._v.from_little_endian()
            }
        }

    }
}

make_primitive_type_from_le!(bool);
make_primitive_type_from_le!(u8);
make_primitive_type_from_le!(i8);
make_primitive_type_from_le!(u16);
make_primitive_type_from_le!(i16);
make_primitive_type_from_le!(u32);
make_primitive_type_from_le!(i32);
make_primitive_type_from_le!(u64);
make_primitive_type_from_le!(i64);
make_primitive_type_from_le!(u128);
make_primitive_type_from_le!(i128);
make_primitive_type_from_le!(usize);
make_primitive_type_from_le!(isize);
make_primitive_type_from_le!(f32);
make_primitive_type_from_le!(f64);
