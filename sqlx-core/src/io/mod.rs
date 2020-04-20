#[macro_use]
mod buf_stream;

mod buf;
mod buf_mut;
mod byte_str;
mod tls;
mod uds;

pub use self::{
    buf::{Buf, ToBuf},
    buf_mut::BufMut,
    buf_stream::BufStream,
    byte_str::ByteStr,
    tls::MaybeTlsStream,
    uds::MaybeUdsStream,
};

#[cfg(test)]
#[doc(hidden)]
macro_rules! bytes (
    ($($b: expr), *) => {{
        use $crate::io::ToBuf;

        let mut buf = Vec::new();
        $(
            buf.extend_from_slice($b.to_buf());
        )*

        buf
    }}
);
