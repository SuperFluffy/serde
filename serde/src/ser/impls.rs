use lib::*;

use ser::{Serialize, SerializeSeq, SerializeTuple, Serializer};

#[cfg(feature = "std")]
use ser::Error;

////////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_impl {
    ($ty:ident, $method:ident $($cast:tt)*) => {
        impl Serialize for $ty {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.$method(*self $($cast)*)
            }
        }
    }
}

primitive_impl!(bool, serialize_bool);
primitive_impl!(isize, serialize_i64 as i64);
primitive_impl!(i8, serialize_i8);
primitive_impl!(i16, serialize_i16);
primitive_impl!(i32, serialize_i32);
primitive_impl!(i64, serialize_i64);
primitive_impl!(usize, serialize_u64 as u64);
primitive_impl!(u8, serialize_u8);
primitive_impl!(u16, serialize_u16);
primitive_impl!(u32, serialize_u32);
primitive_impl!(u64, serialize_u64);
primitive_impl!(f32, serialize_f32);
primitive_impl!(f64, serialize_f64);
primitive_impl!(char, serialize_char);

////////////////////////////////////////////////////////////////////////////////

impl Serialize for str {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl Serialize for String {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for CStr {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.to_bytes())
    }
}

#[cfg(feature = "std")]
impl Serialize for CString {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.to_bytes())
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for PhantomData<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_struct("PhantomData")
    }
}

////////////////////////////////////////////////////////////////////////////////

// Does not require T: Serialize.
impl<T> Serialize for [T; 0] {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        try!(serializer.serialize_seq_fixed_size(0)).end()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Serialize for [T; $len] where T: Serialize {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer,
                {
                    let mut seq = try!(serializer.serialize_seq_fixed_size($len));
                    for e in self {
                        try!(seq.serialize_element(e));
                    }
                    seq.end()
                }
            }
        )+
    }
}

array_impls!(01 02 03 04 05 06 07 08 09 10
             11 12 13 14 15 16 17 18 19 20
             21 22 23 24 25 26 27 28 29 30
             31 32);

////////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for [T]
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        serializer.collect_seq(self)
    }
}

macro_rules! seq_impl {
    ($ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound:ident)* >) => {
        impl<T $(, $typaram)*> Serialize for $ty<T $(, $typaram)*>
        where
            T: Serialize $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                serializer.collect_seq(self)
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(BinaryHeap<T: Ord>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(BTreeSet<T: Ord>);

#[cfg(feature = "std")]
seq_impl!(HashSet<T: Eq + Hash, H: BuildHasher>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(LinkedList<T>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(Vec<T>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(VecDeque<T>);

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<Idx> Serialize for ops::Range<Idx>
where
    Idx: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Range", 2));
        try!(state.serialize_field("start", &self.start));
        try!(state.serialize_field("end", &self.end));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> Serialize for ($($name,)+)
            where
                $($name: Serialize,)+
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = try!(serializer.serialize_tuple($len));
                    $(
                        try!(tuple.serialize_element(&self.$n));
                    )+
                    tuple.end()
                }
            }
        )+
    }
}

tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! map_impl {
    ($ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound:ident)* >) => {
        impl<K, V $(, $typaram)*> Serialize for $ty<K, V $(, $typaram)*>
        where
            K: Serialize $(+ $kbound1 $(+ $kbound2)*)*,
            V: Serialize,
            $($typaram: $bound,)*
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                serializer.collect_seq(self)
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
map_impl!(BTreeMap<K: Ord, V>);

#[cfg(feature = "std")]
map_impl!(HashMap<K: Eq + Hash, V, H: BuildHasher>);

////////////////////////////////////////////////////////////////////////////////

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+ {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                (**self).serialize(serializer)
            }
        }
    };
}

deref_impl!(<'a, T: ?Sized> Serialize for &'a T where T: Serialize);
deref_impl!(<'a, T: ?Sized> Serialize for &'a mut T where T: Serialize);

#[cfg(any(feature = "std", feature = "alloc"))]
deref_impl!(<T: ?Sized> Serialize for Box<T> where T: Serialize);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
deref_impl!(<T> Serialize for Rc<T> where T: Serialize);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
deref_impl!(<T> Serialize for Arc<T> where T: Serialize);

#[cfg(any(feature = "std", feature = "collections"))]
deref_impl!(<'a, T: ?Sized> Serialize for Cow<'a, T> where T: Serialize + ToOwned);

////////////////////////////////////////////////////////////////////////////////

impl<T, E> Serialize for Result<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Ok(ref value) => serializer.serialize_newtype_variant("Result", 0, "Ok", value),
            Result::Err(ref value) => {
                serializer.serialize_newtype_variant("Result", 1, "Err", value)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Duration", 2));
        try!(state.serialize_field("secs", &self.as_secs()));
        try!(state.serialize_field("nanos", &self.subsec_nanos()));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Seralize the `$value` that implements Display as a string,
/// when that string is statically known to never have more than
/// a constant `$MAX_LEN` bytes.
///
/// Panics if the Display impl tries to write more than `$MAX_LEN` bytes.
#[cfg(feature = "std")]
macro_rules! serialize_display_bounded_length {
    ($value: expr, $MAX_LEN: expr, $serializer: expr) => {
        {
            let mut buffer: [u8; $MAX_LEN] = unsafe { mem::uninitialized() };
            let remaining_len;
            {
                let mut remaining = &mut buffer[..];
                write!(remaining, "{}", $value).unwrap();
                remaining_len = remaining.len()
            }
            let written_len = buffer.len() - remaining_len;
            let written = &buffer[..written_len];

            // write! only provides fmt::Formatter to Display implementations,
            // which has methods write_str and write_char but no method to write arbitrary bytes.
            // Therefore, `written` is well-formed in UTF-8.
            let written_str = unsafe {
                str::from_utf8_unchecked(written)
            };
            $serializer.serialize_str(written_str)
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for net::IpAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            net::IpAddr::V4(ref a) => a.serialize(serializer),
            net::IpAddr::V6(ref a) => a.serialize(serializer),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for net::Ipv4Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /// "101.102.103.104".len()
        const MAX_LEN: usize = 15;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

#[cfg(feature = "std")]
impl Serialize for net::Ipv6Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /// "1000:1002:1003:1004:1005:1006:1007:1008".len()
        const MAX_LEN: usize = 39;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for net::SocketAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            net::SocketAddr::V4(ref addr) => addr.serialize(serializer),
            net::SocketAddr::V6(ref addr) => addr.serialize(serializer),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for net::SocketAddrV4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /// "101.102.103.104:65000".len()
        const MAX_LEN: usize = 21;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

#[cfg(feature = "std")]
impl Serialize for net::SocketAddrV6 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /// "[1000:1002:1003:1004:1005:1006:1007:1008]:65000".len()
        const MAX_LEN: usize = 47;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for path::Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.to_str() {
            Some(s) => s.serialize(serializer),
            None => Err(Error::custom("path contains invalid UTF-8 characters")),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for path::PathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_path().serialize(serializer)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl Serialize for OsStr {
    #[cfg(unix)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::os::unix::ffi::OsStrExt;
        serializer.serialize_newtype_variant("OsString", 0, "Unix", self.as_bytes())
    }
    #[cfg(windows)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::os::windows::ffi::OsStrExt;
        let val = self.encode_wide().collect::<Vec<_>>();
        serializer.serialize_newtype_variant("OsString", 1, "Windows", &val)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg(feature = "std")]
impl Serialize for OsString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_os_str().serialize(serializer)
    }
}

#[cfg(feature = "unstable")]
impl<T> Serialize for NonZero<T>
where
    T: Serialize + Zeroable,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).serialize(serializer)
    }
}
