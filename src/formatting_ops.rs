//! Implementations for formatting the various types.
use core::fmt::{Binary, Display, Formatter, LowerHex, Octal, Result, UpperHex};

use super::*;

impl<T: UpperHex + SpecificEndian<T>> UpperHex for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
    }
}
impl<T: UpperHex + SpecificEndian<T>> UpperHex for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: LowerHex + SpecificEndian<T>> LowerHex for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: LowerHex + SpecificEndian<T>> LowerHex for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Octal + SpecificEndian<T>> Octal for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Octal + SpecificEndian<T>> Octal for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Binary + SpecificEndian<T>> Binary for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Binary + SpecificEndian<T>> Binary for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Display + SpecificEndian<T>> Display for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Display + SpecificEndian<T>> Display for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_native()) // delegate to i32's implementation
    }
}
