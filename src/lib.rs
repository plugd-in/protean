//! # `protean`
//! A flexible data type with [serde] support (`serde` feature).
//!
//! ## Why?
//! I found myself making a similar data type quite often
//! for serialized communication of flexible values.
//!
//! **Note:** A lot of serialization formats are not "self-describing"
//! and commonly use integer discriminants for enum variants. This
//! `crate` will make sure to never reorder variants.

#[cfg(test)]
use assert_order::VariantOrder;
use std::borrow::Cow;

const F32_MAX_EXACT_INTEGER: i32 = (1 << f32::MANTISSA_DIGITS) - 1;
const F32_MIN_EXACT_INTEGER: i32 = -F32_MAX_EXACT_INTEGER;

const F64_MAX_EXACT_INTEGER: i64 = (1 << f64::MANTISSA_DIGITS) - 1;
const F64_MIN_EXACT_INTEGER: i64 = -F64_MAX_EXACT_INTEGER;

/// A flexible, fundamental unit of data.
// * **NEVER reorder variants.**
// * **NEVER remove variants.**
// * **ALWAYS add new variants at the END.**
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(VariantOrder))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DataCell<'d> {
    /// Empty cell.
    Empty,

    /* --- Slice types --- */
    /// UTF-8 Text
    Text(Cow<'d, str>),
    /// Raw binary blob suitable for serializing
    /// custom data into RRD data.
    Bytes(Cow<'d, [u8]>),

    /* --- Unsigned integers --- */
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Unsigned64(u64),
    Unsigned128(u128),

    /* --- Signed integers --- */
    Signed8(i8),
    Signed16(i16),
    Signed32(i32),
    Signed64(i64),
    Signed128(i128),

    /* --- Floats --- */
    Float32(f32),
    Float64(f64),
}

impl<'d> DataCell<'d> {
    /// Clones borrowed data and returns a static (unused) lifetime.
    ///
    /// This is different from [DataCell]'s implementation of [Clone],
    /// which copies the underlying shared reference/lifetime.
    pub fn cloned(&self) -> DataCell<'static> {
        match self {
            Self::Empty => DataCell::Empty,
            Self::Text(text) => DataCell::Text(Cow::Owned(text.to_string())),
            Self::Bytes(bytes) => DataCell::Bytes(Cow::Owned(bytes.to_vec())),
            Self::Unsigned8(num) => DataCell::u8(*num),
            Self::Unsigned16(num) => DataCell::u16(*num),
            Self::Unsigned32(num) => DataCell::u32(*num),
            Self::Unsigned64(num) => DataCell::u64(*num),
            Self::Unsigned128(num) => DataCell::u128(*num),
            Self::Signed8(num) => DataCell::i8(*num),
            Self::Signed16(num) => DataCell::i16(*num),
            Self::Signed32(num) => DataCell::i32(*num),
            Self::Signed64(num) => DataCell::i64(*num),
            Self::Signed128(num) => DataCell::i128(*num),
            Self::Float32(num) => DataCell::f32(*num),
            Self::Float64(num) => DataCell::f64(*num),
        }
    }

    /// Create the [Empty](DataCell::Empty) variant.
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Create the [Text](DataCell::Text) variant.
    pub fn text(value: impl Into<Cow<'d, str>>) -> Self {
        Self::Text(value.into())
    }

    /// Returns [Some] if this is a [Text](DataCell::Text) variant.
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Text](DataCell::Text) variant
    /// or a [Bytes](DataCell::Bytes) variant that can be represented
    /// as text.
    pub fn try_as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Bytes(bytes) => str::from_utf8(bytes).ok(),
            _ => None,
        }
    }

    /// Create the [Bytes](DataCell::Bytes) variant.
    pub fn bytes(value: impl Into<Cow<'d, [u8]>>) -> Self {
        Self::Bytes(value.into())
    }

    /// Returns [Some] if this is a [Bytes](DataCell::Bytes) variant.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        if let Self::Bytes(bytes) = self {
            Some(bytes)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Bytes](DataCell::Bytes) variant
    /// or a [Text](DataCell::Text) variant, which will be cast as
    /// a byte slice.
    pub fn try_as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(bytes) => Some(bytes),
            Self::Text(text) => Some(text.as_bytes()),
            _ => None,
        }
    }

    /// Create the [Unsigned8](DataCell::Unsigned8) variant.
    pub fn u8(value: u8) -> Self {
        Self::Unsigned8(value)
    }

    /// Returns [Some] if this is an [Unsigned8](DataCell::Unsigned8) variant.
    pub fn as_u8(&self) -> Option<u8> {
        if let Self::Unsigned8(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is an [Unsigned8](DataCell::Unsigned8) variant
    /// or if this is another number type that can be losslessly represented
    /// as a [u8].
    pub fn try_as_u8(&self) -> Option<u8> {
        match self {
            Self::Unsigned8(num) => Some(*num),
            Self::Unsigned16(num) => u8::try_from(*num).ok(),
            Self::Unsigned32(num) => u8::try_from(*num).ok(),
            Self::Unsigned64(num) => u8::try_from(*num).ok(),
            Self::Unsigned128(num) => u8::try_from(*num).ok(),
            Self::Signed8(num) => u8::try_from(*num).ok(),
            Self::Signed16(num) => u8::try_from(*num).ok(),
            Self::Signed32(num) => u8::try_from(*num).ok(),
            Self::Signed64(num) => u8::try_from(*num).ok(),
            Self::Signed128(num) => u8::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= 0.0 && num <= 255.0 && num.trunc() == num {
                    Some(num as u8)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= 0.0 && num <= 255.0 && num.trunc() == num {
                    Some(num as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Unsigned16](DataCell::Unsigned16) variant.
    pub fn u16(value: u16) -> Self {
        Self::Unsigned16(value)
    }

    /// Returns [Some] if this is an [Unsigned16](DataCell::Unsigned16) variant.
    pub fn as_u16(&self) -> Option<u16> {
        if let Self::Unsigned16(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is an [Unsigned16](DataCell::Unsigned16) variant
    /// or if this is another number type that can be losslessly represented
    /// as a [u16].
    pub fn try_as_u16(&self) -> Option<u16> {
        match self {
            Self::Unsigned8(num) => Some(*num as u16),
            Self::Unsigned16(num) => Some(*num),
            Self::Unsigned32(num) => u16::try_from(*num).ok(),
            Self::Unsigned64(num) => u16::try_from(*num).ok(),
            Self::Unsigned128(num) => u16::try_from(*num).ok(),
            Self::Signed8(num) => u16::try_from(*num).ok(),
            Self::Signed16(num) => u16::try_from(*num).ok(),
            Self::Signed32(num) => u16::try_from(*num).ok(),
            Self::Signed64(num) => u16::try_from(*num).ok(),
            Self::Signed128(num) => u16::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= 0.0 && num <= (u16::MAX as f32) && num.trunc() == num {
                    Some(num as u16)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= 0.0 && num <= (u16::MAX as f64) && num.trunc() == num {
                    Some(num as u16)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Unsigned32](DataCell::Unsigned32) variant.
    pub fn u32(value: u32) -> Self {
        Self::Unsigned32(value)
    }

    /// Returns [Some] if this is an [Unsigned32](DataCell::Unsigned32) variant.
    pub fn as_u32(&self) -> Option<u32> {
        if let Self::Unsigned32(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is an [Unsigned32](DataCell::Unsigned32) variant
    /// or if this is another number type that can be losslessly represented
    /// as a [u32].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned8(num) => Some(*num as u32),
            Self::Unsigned16(num) => Some(*num as u32),
            Self::Unsigned32(num) => Some(*num),
            Self::Unsigned64(num) => u32::try_from(*num).ok(),
            Self::Unsigned128(num) => u32::try_from(*num).ok(),
            Self::Signed8(num) => u32::try_from(*num).ok(),
            Self::Signed16(num) => u32::try_from(*num).ok(),
            Self::Signed32(num) => u32::try_from(*num).ok(),
            Self::Signed64(num) => u32::try_from(*num).ok(),
            Self::Signed128(num) => u32::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= 0.0 && num <= (F32_MAX_EXACT_INTEGER as f32) && num.trunc() == num {
                    Some(num as u32)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= 0.0 && num <= (u32::MAX as f64) && num.trunc() == num {
                    Some(num as u32)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Unsigned64](DataCell::Unsigned64) variant.
    pub fn u64(value: u64) -> Self {
        Self::Unsigned64(value)
    }

    /// Returns [Some] if this is an [Unsigned64](DataCell::Unsigned64) variant.
    pub fn as_u64(&self) -> Option<u64> {
        if let Self::Unsigned64(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is an [Unsigned64](DataCell::Unsigned64) variant
    /// or if this is another number type that can be losslessly represented
    /// as a [u64].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_u64(&self) -> Option<u64> {
        match self {
            Self::Unsigned8(num) => Some(*num as u64),
            Self::Unsigned16(num) => Some(*num as u64),
            Self::Unsigned32(num) => Some(*num as u64),
            Self::Unsigned64(num) => Some(*num),
            Self::Unsigned128(num) => u64::try_from(*num).ok(),
            Self::Signed8(num) => u64::try_from(*num).ok(),
            Self::Signed16(num) => u64::try_from(*num).ok(),
            Self::Signed32(num) => u64::try_from(*num).ok(),
            Self::Signed64(num) => u64::try_from(*num).ok(),
            Self::Signed128(num) => u64::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= 0.0 && num <= (F32_MAX_EXACT_INTEGER as f32) && num.trunc() == num {
                    Some(num as u64)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= 0.0 && num <= (F64_MAX_EXACT_INTEGER as f64) && num.trunc() == num {
                    Some(num as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Unsigned128](DataCell::Unsigned128) variant.
    pub fn u128(value: u128) -> Self {
        Self::Unsigned128(value)
    }

    /// Returns [Some] if this is an [Unsigned128](DataCell::Unsigned128) variant.
    pub fn as_u128(&self) -> Option<u128> {
        if let Self::Unsigned128(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is an [Unsigned128](DataCell::Unsigned128) variant
    /// or if this is another number type that can be losslessly represented
    /// as a [u128].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_u128(&self) -> Option<u128> {
        match self {
            Self::Unsigned8(num) => Some(*num as u128),
            Self::Unsigned16(num) => Some(*num as u128),
            Self::Unsigned32(num) => Some(*num as u128),
            Self::Unsigned64(num) => Some(*num as u128),
            Self::Unsigned128(num) => Some(*num),
            Self::Signed8(num) => u128::try_from(*num).ok(),
            Self::Signed16(num) => u128::try_from(*num).ok(),
            Self::Signed32(num) => u128::try_from(*num).ok(),
            Self::Signed64(num) => u128::try_from(*num).ok(),
            Self::Signed128(num) => u128::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= 0.0 && num <= (F32_MAX_EXACT_INTEGER as f32) && num.trunc() == num {
                    Some(num as u128)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= 0.0 && num <= (F64_MAX_EXACT_INTEGER as f64) && num.trunc() == num {
                    Some(num as u128)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Signed8](DataCell::Signed8) variant.
    pub fn i8(value: i8) -> Self {
        Self::Signed8(value)
    }

    /// Returns [Some] if this is a [Signed8](DataCell::Signed8) variant.
    pub fn as_i8(&self) -> Option<i8> {
        if let Self::Signed8(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Signed8](DataCell::Signed8) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [i8].
    pub fn try_as_i8(&self) -> Option<i8> {
        match self {
            Self::Signed8(num) => Some(*num),
            Self::Signed16(num) => i8::try_from(*num).ok(),
            Self::Signed32(num) => i8::try_from(*num).ok(),
            Self::Signed64(num) => i8::try_from(*num).ok(),
            Self::Signed128(num) => i8::try_from(*num).ok(),
            Self::Unsigned8(num) => i8::try_from(*num).ok(),
            Self::Unsigned16(num) => i8::try_from(*num).ok(),
            Self::Unsigned32(num) => i8::try_from(*num).ok(),
            Self::Unsigned64(num) => i8::try_from(*num).ok(),
            Self::Unsigned128(num) => i8::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= (i8::MIN as f32) && num <= (i8::MAX as f32) && num.trunc() == num {
                    Some(num as i8)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= (i8::MIN as f64) && num <= (i8::MAX as f64) && num.trunc() == num {
                    Some(num as i8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Signed16](DataCell::Signed16) variant.
    pub fn i16(value: i16) -> Self {
        Self::Signed16(value)
    }

    /// Returns [Some] if this is a [Signed16](DataCell::Signed16) variant.
    pub fn as_i16(&self) -> Option<i16> {
        if let Self::Signed16(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Signed16](DataCell::Signed16) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [i16].
    pub fn try_as_i16(&self) -> Option<i16> {
        match self {
            Self::Signed8(num) => Some(*num as i16),
            Self::Signed16(num) => Some(*num),
            Self::Signed32(num) => i16::try_from(*num).ok(),
            Self::Signed64(num) => i16::try_from(*num).ok(),
            Self::Signed128(num) => i16::try_from(*num).ok(),
            Self::Unsigned8(num) => i16::try_from(*num).ok(),
            Self::Unsigned16(num) => i16::try_from(*num).ok(),
            Self::Unsigned32(num) => i16::try_from(*num).ok(),
            Self::Unsigned64(num) => i16::try_from(*num).ok(),
            Self::Unsigned128(num) => i16::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= (i16::MIN as f32) && num <= (i16::MAX as f32) && num.trunc() == num {
                    Some(num as i16)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= (i16::MIN as f64) && num <= (i16::MAX as f64) && num.trunc() == num {
                    Some(num as i16)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Signed32](DataCell::Signed32) variant.
    pub fn i32(value: i32) -> Self {
        Self::Signed32(value)
    }

    /// Returns [Some] if this is a [Signed32](DataCell::Signed32) variant.
    pub fn as_i32(&self) -> Option<i32> {
        if let Self::Signed32(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Signed32](DataCell::Signed32) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [i32].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_i32(&self) -> Option<i32> {
        match self {
            Self::Signed8(num) => Some(*num as i32),
            Self::Signed16(num) => Some(*num as i32),
            Self::Signed32(num) => Some(*num),
            Self::Signed64(num) => i32::try_from(*num).ok(),
            Self::Signed128(num) => i32::try_from(*num).ok(),
            Self::Unsigned8(num) => i32::try_from(*num).ok(),
            Self::Unsigned16(num) => i32::try_from(*num).ok(),
            Self::Unsigned32(num) => i32::try_from(*num).ok(),
            Self::Unsigned64(num) => i32::try_from(*num).ok(),
            Self::Unsigned128(num) => i32::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= (F32_MIN_EXACT_INTEGER as f32)
                    && num <= (F32_MAX_EXACT_INTEGER as f32)
                    && num.trunc() == num
                {
                    Some(num as i32)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= (i32::MIN as f64) && num <= (i32::MAX as f64) && num.trunc() == num {
                    Some(num as i32)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Signed64](DataCell::Signed64) variant.
    pub fn i64(value: i64) -> Self {
        Self::Signed64(value)
    }

    /// Returns [Some] if this is a [Signed64](DataCell::Signed64) variant.
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::Signed64(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Signed64](DataCell::Signed64) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [i64].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_i64(&self) -> Option<i64> {
        match self {
            Self::Signed8(num) => Some(*num as i64),
            Self::Signed16(num) => Some(*num as i64),
            Self::Signed32(num) => Some(*num as i64),
            Self::Signed64(num) => Some(*num),
            Self::Signed128(num) => i64::try_from(*num).ok(),
            Self::Unsigned8(num) => i64::try_from(*num).ok(),
            Self::Unsigned16(num) => i64::try_from(*num).ok(),
            Self::Unsigned32(num) => i64::try_from(*num).ok(),
            Self::Unsigned64(num) => i64::try_from(*num).ok(),
            Self::Unsigned128(num) => i64::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= (F32_MIN_EXACT_INTEGER as f32)
                    && num <= (F32_MAX_EXACT_INTEGER as f32)
                    && num.trunc() == num
                {
                    Some(num as i64)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= (F64_MIN_EXACT_INTEGER as f64)
                    && num <= (F64_MAX_EXACT_INTEGER as f64)
                    && num.trunc() == num
                {
                    Some(num as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Signed128](DataCell::Signed128) variant.
    pub fn i128(value: i128) -> Self {
        Self::Signed128(value)
    }

    /// Returns [Some] if this is a [Signed128](DataCell::Signed128) variant.
    pub fn as_i128(&self) -> Option<i128> {
        if let Self::Signed128(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Signed128](DataCell::Signed128) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [i128].
    ///
    /// **Note:** Floats are only returned when in the range where there aren't
    /// any gaps in the integers that can be represented as a float.
    pub fn try_as_i128(&self) -> Option<i128> {
        match self {
            Self::Signed8(num) => Some(*num as i128),
            Self::Signed16(num) => Some(*num as i128),
            Self::Signed32(num) => Some(*num as i128),
            Self::Signed64(num) => Some(*num as i128),
            Self::Signed128(num) => Some(*num),
            Self::Unsigned8(num) => i128::try_from(*num).ok(),
            Self::Unsigned16(num) => i128::try_from(*num).ok(),
            Self::Unsigned32(num) => i128::try_from(*num).ok(),
            Self::Unsigned64(num) => i128::try_from(*num).ok(),
            Self::Unsigned128(num) => i128::try_from(*num).ok(),
            Self::Float32(num) => {
                let num = *num;

                if num >= (F32_MIN_EXACT_INTEGER as f32)
                    && num <= (F32_MAX_EXACT_INTEGER as f32)
                    && num.trunc() == num
                {
                    Some(num as i128)
                } else {
                    None
                }
            }
            Self::Float64(num) => {
                let num = *num;

                if num >= (F64_MIN_EXACT_INTEGER as f64)
                    && num <= (F64_MAX_EXACT_INTEGER as f64)
                    && num.trunc() == num
                {
                    Some(num as i128)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Create the [Float32](DataCell::Float32) variant.
    pub fn f32(value: f32) -> Self {
        Self::Float32(value)
    }

    /// Returns [Some] if this is a [Float32](DataCell::Float32) variant.
    pub fn as_f32(&self) -> Option<f32> {
        if let Self::Float32(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Float32](DataCell::Float32) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [f32].
    ///
    /// **Note:** [f64] is not returned as f32, due to differences in precision.
    /// Likewise, integers greater than 16 bits will generally not be returned
    /// as an f32 due to gaps. So, `255u32` will be returned, but
    /// [u32::MAX] will not be returned as it exceeds [u16::MAX].
    pub fn try_as_f32(&self) -> Option<f32> {
        match self {
            Self::Float32(num) => Some(*num),
            Self::Float64(_num) => None,
            Self::Unsigned8(num) => Some(f32::from(*num)),
            Self::Unsigned16(num) => Some(f32::from(*num)),
            Self::Unsigned32(num) => u16::try_from(*num).ok().map(f32::from),
            Self::Unsigned64(num) => u16::try_from(*num).ok().map(f32::from),
            Self::Unsigned128(num) => u16::try_from(*num).ok().map(f32::from),
            Self::Signed8(num) => Some(f32::from(*num)),
            Self::Signed16(num) => Some(f32::from(*num)),
            Self::Signed32(num) => i16::try_from(*num).ok().map(f32::from),
            Self::Signed64(num) => i16::try_from(*num).ok().map(f32::from),
            Self::Signed128(num) => i16::try_from(*num).ok().map(f32::from),
            _ => None,
        }
    }

    /// Create the [Float64](DataCell::Float64) variant.
    pub fn f64(value: f64) -> Self {
        Self::Float64(value)
    }

    /// Returns [Some] if this is a [Float64](DataCell::Float64) variant.
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Float64(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    /// Returns [Some] if this is a [Float64](DataCell::Float64) variant
    /// or if this is another number type that can be losslessly represented
    /// as an [f64].
    ///
    /// **Note:** Integers greater than 16 bits will generally not be
    /// returned as an [f32] due to gaps. So, `255u32` will be returned, but
    /// [u32::MAX] will not be returned as it exceeds [u16::MAX].
    pub fn try_as_f64(&self) -> Option<f64> {
        match self {
            Self::Float32(num) => Some(f64::from(*num)),
            Self::Float64(num) => Some(*num),
            Self::Unsigned8(num) => Some(f64::from(*num)),
            Self::Unsigned16(num) => Some(f64::from(*num)),
            Self::Unsigned32(num) => Some(f64::from(*num)),
            Self::Unsigned64(num) => u32::try_from(*num).ok().map(f64::from),
            Self::Unsigned128(num) => u32::try_from(*num).ok().map(f64::from),
            Self::Signed8(num) => Some(f64::from(*num)),
            Self::Signed16(num) => Some(f64::from(*num)),
            Self::Signed32(num) => Some(f64::from(*num)),
            Self::Signed64(num) => i32::try_from(*num).ok().map(f64::from),
            Self::Signed128(num) => i32::try_from(*num).ok().map(f64::from),
            _ => None,
        }
    }
}

impl From<()> for DataCell<'static> {
    fn from(_value: ()) -> Self {
        Self::Empty
    }
}

impl From<String> for DataCell<'static> {
    fn from(value: String) -> Self {
        Self::Text(Cow::from(value))
    }
}

impl From<Box<str>> for DataCell<'static> {
    fn from(value: Box<str>) -> Self {
        Self::Text(Cow::from(value.into_string()))
    }
}

impl<'a> From<&'a str> for DataCell<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(Cow::Borrowed(value))
    }
}

impl From<Vec<u8>> for DataCell<'static> {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Cow::from(value))
    }
}

impl From<Box<[u8]>> for DataCell<'static> {
    fn from(value: Box<[u8]>) -> Self {
        Self::Bytes(Cow::from(value.into_vec()))
    }
}

impl<'a> From<&'a [u8]> for DataCell<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Bytes(Cow::Borrowed(value))
    }
}

impl From<u8> for DataCell<'static> {
    fn from(value: u8) -> Self {
        Self::Unsigned8(value)
    }
}

impl From<u16> for DataCell<'static> {
    fn from(value: u16) -> Self {
        Self::Unsigned16(value)
    }
}

impl From<u32> for DataCell<'static> {
    fn from(value: u32) -> Self {
        Self::Unsigned32(value)
    }
}

impl From<u64> for DataCell<'static> {
    fn from(value: u64) -> Self {
        Self::Unsigned64(value)
    }
}

impl From<u128> for DataCell<'static> {
    fn from(value: u128) -> Self {
        Self::Unsigned128(value)
    }
}

impl From<i8> for DataCell<'static> {
    fn from(value: i8) -> Self {
        Self::Signed8(value)
    }
}

impl From<i16> for DataCell<'static> {
    fn from(value: i16) -> Self {
        Self::Signed16(value)
    }
}

impl From<i32> for DataCell<'static> {
    fn from(value: i32) -> Self {
        Self::Signed32(value)
    }
}

impl From<i64> for DataCell<'static> {
    fn from(value: i64) -> Self {
        Self::Signed64(value)
    }
}

impl From<i128> for DataCell<'static> {
    fn from(value: i128) -> Self {
        Self::Signed128(value)
    }
}

impl From<f32> for DataCell<'static> {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for DataCell<'static> {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::DataCell;
    use assert_order::assert_order;

    #[test]
    fn proper_order() {
        // **NOTE:** DO NOT REORDER THESE!
        //
        // This test ensures that no issues arise from reordering,
        // maintaining backward compatibility.
        assert_order::<DataCell, _, _>([
            /* Empty */
            "Empty",
            /* Slice types */
            "Text",
            "Bytes",
            /* Unsigned integers */
            "Unsigned8",
            "Unsigned16",
            "Unsigned32",
            "Unsigned64",
            "Unsigned128",
            /* Signed integers */
            "Signed8",
            "Signed16",
            "Signed32",
            "Signed64",
            "Signed128",
            /* Floats */
            "Float32",
            "Float64",
        ]);
    }
}
