use std::{
    borrow::Cow,
    io::{Read, Write},
};

pub trait CodecRead: Sized {
    fn codec_read(reader: &mut impl Read) -> std::io::Result<Self>;
}
pub trait CodecWrite: Sized {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()>;
    fn codec_len(&self) -> usize;
}

macro_rules! codec_primitive {
    ($t:ty) => {
        impl CodecRead for $t {
            #[inline]
            fn codec_read(reader: &mut impl Read) -> std::io::Result<Self> {
                let mut buf = [0u8; size_of::<Self>()];
                reader.read_exact(buf.as_mut_slice())?;
                Ok(Self::from_be_bytes(buf))
            }
        }
        impl CodecWrite for $t {
            #[inline]
            fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
                writer.write_all(&self.to_be_bytes())
            }
            #[inline]
            fn codec_len(&self) -> usize {
                size_of::<Self>()
            }
        }
    };
}
codec_primitive!(i32);
codec_primitive!(u32);
codec_primitive!(u16);
codec_primitive!(f32);
codec_primitive!(f64);

impl CodecRead for () {
    #[inline]
    fn codec_read(_reader: &mut impl Read) -> std::io::Result<Self> {
        Ok(())
    }
}
impl CodecWrite for () {
    #[inline]
    fn codec_write(&self, _writer: &mut impl Write) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn codec_len(&self) -> usize {
        0
    }
}

impl CodecRead for String {
    fn codec_read(reader: &mut impl Read) -> std::io::Result<Self> {
        let len = i32::codec_read(reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(std::io::Error::other)
    }
}
impl CodecWrite for String {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        (self.len() as i32).codec_write(writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline]
    fn codec_len(&self) -> usize {
        size_of::<i32>() + self.len()
    }
}
impl<T: CodecRead> CodecRead for Vec<T> {
    fn codec_read(reader: &mut impl Read) -> std::io::Result<Self> {
        (0..i32::codec_read(reader)?)
            .map(|_| T::codec_read(reader))
            .collect()
    }
}
impl<T: CodecWrite> CodecWrite for Vec<T> {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        (self.len() as i32).codec_write(writer)?;
        for v in self {
            v.codec_write(writer)?;
        }
        Ok(())
    }
    #[inline]
    fn codec_len(&self) -> usize {
        size_of::<i32>() + self.iter().map(T::codec_len).sum::<usize>()
    }
}
#[derive(Debug)]
pub struct Vec2D<T> {
    pub size: [usize; 2],
    pub data: Vec<T>,
}
impl<T> Default for Vec2D<T> {
    fn default() -> Self {
        Self {
            size: Default::default(),
            data: Vec::new(),
        }
    }
}
impl<T: CodecRead> CodecRead for Vec2D<T> {
    fn codec_read(reader: &mut impl Read) -> std::io::Result<Self> {
        let size = [usize::codec_read(reader)?, usize::codec_read(reader)?];
        let data = (0..size[0] * size[1])
            .map(|_| T::codec_read(reader))
            .collect::<std::io::Result<_>>()?;
        Ok(Self { size, data })
    }
}
impl<T: CodecWrite> CodecWrite for Vec2D<T> {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        (self.size[0]).codec_write(writer)?;
        (self.size[1]).codec_write(writer)?;
        self.data.iter().map(|v| v.codec_write(writer)).collect()
    }
    #[inline]
    fn codec_len(&self) -> usize {
        usize::codec_len(&0) * 2 + self.data.iter().map(T::codec_len).sum::<usize>()
    }
}

#[derive(Debug)]
pub struct FixedString<'s, const N: usize> {
    pub inner: Cow<'s, str>,
}
impl<const N: usize> CodecRead for FixedString<'_, N> {
    fn codec_read(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut name_buf = [0u8; N];
        reader.read_exact(&mut name_buf)?;
        let name_len = name_buf.iter().position(|b| *b == 0).unwrap_or(N);
        let name = str::from_utf8(&name_buf[..name_len])
            .map_err(std::io::Error::other)?
            .to_string();
        Ok(Self { inner: name.into() })
    }
}
impl<const N: usize> CodecWrite for FixedString<'_, N> {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let mut name_buf = [0u8; N];
        name_buf[0..self.inner.len()].copy_from_slice(self.inner.as_bytes());
        writer.write_all(&name_buf)?;
        Ok(())
    }

    fn codec_len(&self) -> usize {
        N
    }
}

#[derive(Debug)]
#[apply(CodecReadDerive)]
#[apply(CodecWriteDerive)]
pub(crate) struct Header {
    pub name: FixedString<'static, 32>,
    pub body_len: i32,
    pub response: u16,
    _pad: u16,
}
impl Header {
    pub fn new(name: &'static str, body_len: usize) -> Self {
        Self {
            name: FixedString { inner: name.into() },
            body_len: body_len as i32,
            response: 1,
            _pad: 0,
        }
    }
}

#[derive(Debug)]
#[apply(CodecReadDerive)]
pub(crate) struct Footer {
    pub status: u32,
    pub description: String,
}
impl Footer {
    pub fn into_result(self) -> NanonisTcpResult<()> {
        match self.status {
            0 => Ok(()),
            _ => Err(NanonisTcpError::Api(self.description)),
        }
    }
}

macro_rules! CodecWriteDerive {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        // Reproduce the original struct unchanged
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field: $ty
            ),*
        }

        // Now do whatever you want with names + types
        impl crate::CodecWrite for $name {
            fn codec_write(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                $(crate::CodecWrite::codec_write(&self.$field, writer)?;)+
                Ok(())
            }

            fn codec_len(&self) -> usize {
                0 $(+ crate::CodecWrite::codec_len(&self.$field))+
            }
        }
    };
}
pub(crate) use CodecWriteDerive;

macro_rules! CodecReadDerive {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        // Reproduce the original struct unchanged
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field: $ty
            ),*
        }

        // Now do whatever you want with names + types
        impl crate::CodecRead for $name {
            fn codec_read(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
                $(let $field = crate::CodecRead::codec_read(reader)?;)+
                Ok(Self {
                    $($field),+
                })
            }
        }
    };
}
pub(crate) use CodecReadDerive;
use macro_rules_attribute::apply;

use crate::error::{NanonisTcpError, NanonisTcpResult};
