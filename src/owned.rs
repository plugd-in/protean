use delegate::delegate;
use stable_deref_trait::StableDeref;
use std::{borrow::Cow, ops::Deref, rc::Rc, str::Utf8Error, string::FromUtf8Error, sync::Arc};
use yoke::{CloneableCart, Yoke};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

use crate::DataCell;

#[cfg(feature = "serde")]
fn serialize_yoke<B, S>(
    datacell: &Yoke<DataCell<'static>, Option<B>>,
    ser: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    datacell.get().serialize(ser)
}

#[cfg_attr(feature = "serde", derive(Serialize))]
/// A [DataCell] that borrows from an owned
/// buffer without copying.
pub struct OwnedDataCell<B> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_yoke"))]
    pub(crate) yoke: Yoke<DataCell<'static>, Option<B>>,
}

impl<B> Clone for OwnedDataCell<B>
where
    B: CloneableCart,
{
    fn clone(&self) -> Self {
        Self {
            yoke: self.yoke.clone(),
        }
    }
}

impl<B> std::fmt::Debug for OwnedDataCell<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.yoke.get(), f)
    }
}

impl<B> PartialEq for OwnedDataCell<B> {
    fn eq(&self, other: &Self) -> bool {
        self.yoke.get() == other.yoke.get()
    }
}

impl<B> PartialEq<DataCell<'_>> for OwnedDataCell<B> {
    fn eq(&self, other: &DataCell<'_>) -> bool {
        self.yoke.get() == other
    }
}

impl From<DataCell<'static>> for OwnedDataCell<Cow<'static, [u8]>> {
    fn from(value: DataCell<'static>) -> Self {
        match value {
            DataCell::Empty => OwnedDataCell::empty(),
            DataCell::Text(text) => {
                let cart = match text {
                    Cow::Borrowed(text) => Cow::Borrowed(text.as_bytes()),
                    Cow::Owned(text) => Cow::Owned(text.into_bytes()),
                };

                let yoke = Yoke::<DataCell, _>::attach_to_cart(cart, |text| {
                    // SAFETY: We got str, then turned it into bytes, so it should be valid
                    // to turn it back into str.
                    let text = unsafe { std::str::from_utf8_unchecked(text) };

                    DataCell::Text(Cow::Borrowed(text))
                });
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
            DataCell::Bytes(bytes) => {
                let yoke = Yoke::<DataCell, _>::attach_to_cart(bytes, |bytes| {
                    DataCell::Bytes(Cow::Borrowed(bytes))
                });
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
            DataCell::Unsigned8(value) => OwnedDataCell::u8(value),
            DataCell::Unsigned16(value) => OwnedDataCell::u16(value),
            DataCell::Unsigned32(value) => OwnedDataCell::u32(value),
            DataCell::Unsigned64(value) => OwnedDataCell::u64(value),
            DataCell::Unsigned128(value) => OwnedDataCell::u128(value),
            DataCell::Signed8(value) => OwnedDataCell::i8(value),
            DataCell::Signed16(value) => OwnedDataCell::i16(value),
            DataCell::Signed32(value) => OwnedDataCell::i32(value),
            DataCell::Signed64(value) => OwnedDataCell::i64(value),
            DataCell::Signed128(value) => OwnedDataCell::i128(value),
            DataCell::Float32(value) => OwnedDataCell::f32(value),
            DataCell::Float64(value) => OwnedDataCell::f64(value),
        }
    }
}

macro_rules! from_text_bytes {
    (<$backing_text:ty, $backing_bytes:ty>) => {
        impl From<$backing_text> for OwnedDataCell<$backing_text> {
            /// Constructs a [DataCell::Text] backed by an owned
            /// string buffer.
            fn from(value: $backing_text) -> Self {
                let yoke = Yoke::<DataCell, _>::attach_to_cart(value, |text| DataCell::text(text));
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
        }

        impl From<$backing_bytes> for OwnedDataCell<$backing_bytes> {
            /// Constructs a [DataCell::Bytes] backed by an owned buffer.
            fn from(value: $backing_bytes) -> Self {
                let yoke =
                    Yoke::<DataCell, _>::attach_to_cart(value, |bytes| DataCell::bytes(bytes));
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
        }
    };
}

from_text_bytes!(<&'static str, &'static [u8]>);
from_text_bytes!(<Arc<str>, Arc<[u8]>>);
from_text_bytes!(<Rc<str>, Rc<[u8]>>);
from_text_bytes!(<Box<str>, Box<[u8]>>);
from_text_bytes!(<String, Vec<u8>>);

macro_rules! try_from_bytes_to_text {
    ($ptr:ident) => {
        impl TryFrom<$ptr<[u8]>> for OwnedDataCell<$ptr<str>> {
            type Error = Utf8Error;

            /// Tries to build an owned [DataCell::Text] from bytes.
            ///
            /// **Note:** This does not copy, clone, or allocate.
            fn try_from(value: $ptr<[u8]>) -> Result<Self, Self::Error> {
                let _ = std::str::from_utf8(value.as_ref())?;

                // SAFETY: `str` has the same layout as `[u8]`.
                let string = unsafe { $ptr::from_raw($ptr::into_raw(value) as *const str) };
                let yoke = Yoke::<DataCell, _>::attach_to_cart(string, |text| DataCell::text(text));
                let yoke = yoke.wrap_cart_in_option();

                Ok(Self { yoke })
            }
        }
    };
}

try_from_bytes_to_text!(Arc);
try_from_bytes_to_text!(Rc);

impl TryFrom<Box<[u8]>> for OwnedDataCell<Box<str>> {
    type Error = Utf8Error;

    /// Tries to build an owned [DataCell::Text] from bytes.
    ///
    /// **Note:** This does not copy, clone, or allocate.
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        let _ = std::str::from_utf8(value.as_ref())?;

        let raw = Box::into_raw(value);
        let string = unsafe { Box::from_raw(raw as *mut str) };

        let yoke = Yoke::<DataCell, _>::attach_to_cart(string, |text| DataCell::text(text));
        let yoke = yoke.wrap_cart_in_option();

        Ok(Self { yoke })
    }
}

impl TryFrom<Vec<u8>> for OwnedDataCell<String> {
    type Error = FromUtf8Error;

    /// Tries to build an owned [DataCell::Text] from bytes.
    ///
    /// **Note:** This does not copy, clone, or allocate.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let string = String::from_utf8(value)?;

        let yoke = Yoke::<DataCell, _>::attach_to_cart(string, |text| DataCell::text(text));
        let yoke = yoke.wrap_cart_in_option();

        Ok(Self { yoke })
    }
}

macro_rules! from_text_to_bytes {
    ($ptr:ident) => {
        impl From<$ptr<str>> for OwnedDataCell<$ptr<[u8]>> {
            /// Constructs a [DataCell::Text] backed by the passed
            /// in string buffer, converting the string buffer into
            /// a byte buffer without copying/cloning.
            fn from(value: $ptr<str>) -> Self {
                // From<$ptr<str>> for $ptr<[u8]> essentially just casts
                // the underlying pointer type, as str and [u8] have
                // the same layout.
                let bytes: $ptr<[u8]> = $ptr::from(value);

                let yoke = Yoke::<DataCell, _>::attach_to_cart(bytes, |text| {
                    // SAFETY: Started as a string.
                    let text = unsafe { std::str::from_utf8_unchecked(text) };

                    DataCell::text(text)
                });
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
        }
    };
}

from_text_to_bytes!(Arc);
from_text_to_bytes!(Rc);
from_text_to_bytes!(Box);

impl From<Box<str>> for OwnedDataCell<Vec<u8>> {
    /// Constructs a [DataCell::Text] backed by the passed
    /// in string buffer, converting the string buffer into
    /// a byte buffer without copying/cloning.
    fn from(value: Box<str>) -> Self {
        let bytes = value.into_boxed_bytes().into_vec();

        let yoke = Yoke::<DataCell, _>::attach_to_cart(bytes, |text| {
            // SAFETY: Started as a string.
            let text = unsafe { std::str::from_utf8_unchecked(text) };

            DataCell::text(text)
        });
        let yoke = yoke.wrap_cart_in_option();

        Self { yoke }
    }
}

macro_rules! from_string_to_bytes {
    (<$from:ty, $to:ty>, ($param:ident) => $map:expr) => {
        impl From<$from> for OwnedDataCell<$to> {
            /// Constructs a [DataCell::Text] backed by the passed
            /// in string buffer, converting the string buffer into
            /// a byte buffer without copying/cloning.
            fn from($param: $from) -> Self {
                let bytes: $to = $map;

                let yoke = Yoke::<DataCell, _>::attach_to_cart(bytes, |text| {
                    // SAFETY: Started as a string.
                    let text = unsafe { std::str::from_utf8_unchecked(text) };

                    DataCell::text(text)
                });
                let yoke = yoke.wrap_cart_in_option();

                Self { yoke }
            }
        }
    };
}

from_string_to_bytes! {
    <String, Box<[u8]>>,
    (value) => Box::from(value.into_boxed_str())
}

from_string_to_bytes! {
    <String, Vec<u8>>,
    (value) => value.into_bytes()
}

impl<B> From<()> for OwnedDataCell<B> {
    fn from(_value: ()) -> Self {
        OwnedDataCell::empty()
    }
}

impl<B> From<u8> for OwnedDataCell<B> {
    fn from(value: u8) -> Self {
        OwnedDataCell::u8(value)
    }
}

impl<B> From<u16> for OwnedDataCell<B> {
    fn from(value: u16) -> Self {
        OwnedDataCell::u16(value)
    }
}

impl<B> From<u32> for OwnedDataCell<B> {
    fn from(value: u32) -> Self {
        OwnedDataCell::u32(value)
    }
}

impl<B> From<u64> for OwnedDataCell<B> {
    fn from(value: u64) -> Self {
        OwnedDataCell::u64(value)
    }
}

impl<B> From<u128> for OwnedDataCell<B> {
    fn from(value: u128) -> Self {
        OwnedDataCell::u128(value)
    }
}

impl<B> From<i8> for OwnedDataCell<B> {
    fn from(value: i8) -> Self {
        OwnedDataCell::i8(value)
    }
}

impl<B> From<i16> for OwnedDataCell<B> {
    fn from(value: i16) -> Self {
        OwnedDataCell::i16(value)
    }
}

impl<B> From<i32> for OwnedDataCell<B> {
    fn from(value: i32) -> Self {
        OwnedDataCell::i32(value)
    }
}

impl<B> From<i64> for OwnedDataCell<B> {
    fn from(value: i64) -> Self {
        OwnedDataCell::i64(value)
    }
}

impl<B> From<i128> for OwnedDataCell<B> {
    fn from(value: i128) -> Self {
        OwnedDataCell::i128(value)
    }
}

impl<B> From<f32> for OwnedDataCell<B> {
    fn from(value: f32) -> Self {
        OwnedDataCell::f32(value)
    }
}

impl<B> From<f64> for OwnedDataCell<B> {
    fn from(value: f64) -> Self {
        OwnedDataCell::f64(value)
    }
}

impl<B> OwnedDataCell<B> {
    /// Deserialize a [DataCell] from an owned buffer, without copying,
    /// with the passed in infallible `builder` method.
    ///
    /// If the [DataCell] isn't a variant that borrows data,
    /// this discards the passed in buffer to save memory.
    pub fn build<F>(backing: B, builder: F) -> Self
    where
        B: Deref<Target = [u8]> + 'static,
        B: StableDeref,
        F: for<'b> FnOnce(&'b [u8]) -> DataCell<'b>,
    {
        let yoke = Yoke::<DataCell, _>::attach_to_cart(backing, |cart| builder(cart));
        let yoke = yoke.wrap_cart_in_option();

        // We discard the owned slice if we don't need it.
        match yoke.get() {
            DataCell::Bytes(bytes) => match bytes.clone() {
                Cow::Borrowed(_) => Self { yoke },
                Cow::Owned(bytes) => {
                    let yoke = Yoke::new_owned(DataCell::Bytes(Cow::Owned(bytes)));

                    Self { yoke }
                }
            },
            DataCell::Text(text) => match text.clone() {
                Cow::Borrowed(_) => Self { yoke },
                Cow::Owned(text) => {
                    let yoke = Yoke::new_owned(DataCell::Text(Cow::Owned(text)));

                    Self { yoke }
                }
            },
            data => {
                let data = data.cloned();
                let yoke = Yoke::new_owned(data);

                Self { yoke }
            }
        }
    }

    /// Try to deserialize a [DataCell] from an owned buffer, without copying,
    /// with the passed in fallible `builder` method.
    ///
    /// If the [DataCell] isn't a variant that borrows data,
    /// this discards the passed in buffer to save memory.
    pub fn try_build<F, E>(backing: B, builder: F) -> Result<Self, E>
    where
        B: Deref<Target = [u8]> + 'static,
        B: StableDeref,
        F: for<'b> FnOnce(&'b [u8]) -> Result<DataCell<'b>, E>,
    {
        let yoke = Yoke::<DataCell, _>::try_attach_to_cart(backing, |cart| builder(cart))?;
        let yoke = yoke.wrap_cart_in_option();

        // We discard the owned slice if we don't need it.
        match yoke.get() {
            DataCell::Bytes(bytes) => match bytes.clone() {
                Cow::Borrowed(_) => Ok(Self { yoke }),
                Cow::Owned(bytes) => {
                    let yoke = Yoke::new_owned(DataCell::Bytes(Cow::Owned(bytes)));

                    Ok(Self { yoke })
                }
            },
            DataCell::Text(text) => match text.clone() {
                Cow::Borrowed(_) => Ok(Self { yoke }),
                Cow::Owned(text) => {
                    let yoke = Yoke::new_owned(DataCell::Text(Cow::Owned(text)));

                    Ok(Self { yoke })
                }
            },
            data => {
                let data = data.cloned();
                let yoke = Yoke::new_owned(data);

                Ok(Self { yoke })
            }
        }
    }

    /// Get a borrowed view of the datacell.
    pub fn borrowed(&self) -> &DataCell<'_> {
        self.yoke.get()
    }

    pub fn empty() -> Self {
        let yoke = Yoke::new_owned(DataCell::empty());

        Self { yoke }
    }

    pub fn text<T>(value: T) -> Self
    where
        T: Deref<Target = str>,
        Self: From<T>,
    {
        Self::from(value)
    }

    pub fn bytes<T>(value: T) -> Self
    where
        T: Deref<Target = [u8]>,
        Self: From<T>,
    {
        Self::from(value)
    }

    pub fn u8(value: u8) -> Self {
        let yoke = Yoke::new_owned(DataCell::u8(value));

        Self { yoke }
    }

    pub fn u16(value: u16) -> Self {
        let yoke = Yoke::new_owned(DataCell::u16(value));

        Self { yoke }
    }

    pub fn u32(value: u32) -> Self {
        let yoke = Yoke::new_owned(DataCell::u32(value));

        Self { yoke }
    }

    pub fn u64(value: u64) -> Self {
        let yoke = Yoke::new_owned(DataCell::u64(value));

        Self { yoke }
    }

    pub fn u128(value: u128) -> Self {
        let yoke = Yoke::new_owned(DataCell::u128(value));

        Self { yoke }
    }

    pub fn i8(value: i8) -> Self {
        let yoke = Yoke::new_owned(DataCell::i8(value));

        Self { yoke }
    }

    pub fn i16(value: i16) -> Self {
        let yoke = Yoke::new_owned(DataCell::i16(value));

        Self { yoke }
    }

    pub fn i32(value: i32) -> Self {
        let yoke = Yoke::new_owned(DataCell::i32(value));

        Self { yoke }
    }

    pub fn i64(value: i64) -> Self {
        let yoke = Yoke::new_owned(DataCell::i64(value));

        Self { yoke }
    }

    pub fn i128(value: i128) -> Self {
        let yoke = Yoke::new_owned(DataCell::i128(value));

        Self { yoke }
    }

    pub fn f32(value: f32) -> Self {
        let yoke = Yoke::new_owned(DataCell::f32(value));

        Self { yoke }
    }

    pub fn f64(value: f64) -> Self {
        let yoke = Yoke::new_owned(DataCell::f64(value));

        Self { yoke }
    }

    delegate! {
        to self.yoke.get() {
            pub fn as_text(&self) -> Option<&str>;
            pub fn try_as_text(&self) -> Option<&str>;

            pub fn as_bytes(&self) -> Option<&[u8]>;
            pub fn try_as_bytes(&self) -> Option<&[u8]>;

            pub fn as_u8(&self) -> Option<u8>;
            pub fn try_as_u8(&self) -> Option<u8>;

            pub fn as_u16(&self) -> Option<u16>;
            pub fn try_as_u16(&self) -> Option<u16>;

            pub fn as_u32(&self) -> Option<u32>;
            pub fn try_as_u32(&self) -> Option<u32>;

            pub fn as_u64(&self) -> Option<u64>;
            pub fn try_as_u64(&self) -> Option<u64>;

            pub fn as_u128(&self) -> Option<u128>;
            pub fn try_as_u128(&self) -> Option<u128>;

            pub fn as_i8(&self) -> Option<i8>;
            pub fn try_as_i8(&self) -> Option<i8>;

            pub fn as_i16(&self) -> Option<i16>;
            pub fn try_as_i16(&self) -> Option<i16>;

            pub fn as_i32(&self) -> Option<i32>;
            pub fn try_as_i32(&self) -> Option<i32>;

            pub fn as_i64(&self) -> Option<i64>;
            pub fn try_as_i64(&self) -> Option<i64>;

            pub fn as_i128(&self) -> Option<i128>;
            pub fn try_as_i128(&self) -> Option<i128>;

            pub fn as_f32(&self) -> Option<f32>;
            pub fn try_as_f32(&self) -> Option<f32>;

            pub fn as_f64(&self) -> Option<f64>;
            pub fn try_as_f64(&self) -> Option<f64>;
        }
    }
}
