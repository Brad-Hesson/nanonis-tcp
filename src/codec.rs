use std::io::{Read, Write};

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
        Ok(String::from_utf8(buf).map_err(std::io::Error::other)?)
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
    pub size: [i32; 2],
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
        let rows = i32::codec_read(reader)?;
        let cols = i32::codec_read(reader)?;
        Ok(Self {
            size: [rows, cols],
            data: (0..rows * cols)
                .map(|_| T::codec_read(reader))
                .collect::<std::io::Result<_>>()?,
        })
    }
}
impl<T: CodecWrite> CodecWrite for Vec2D<T> {
    fn codec_write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        (self.size[0]).codec_write(writer)?;
        (self.size[1]).codec_write(writer)?;
        for v in &self.data {
            v.codec_write(writer)?;
        }
        Ok(())
    }
    #[inline]
    fn codec_len(&self) -> usize {
        size_of::<i32>() * 2 + self.data.iter().map(T::codec_len).sum::<usize>()
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
