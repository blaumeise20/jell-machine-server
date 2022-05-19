#[derive(Clone)]
pub struct InputStream {
    pub bytes: Vec<u8>,
}

impl InputStream {
    pub fn new(bytes: Vec<u8>) -> Self {
        InputStream {
            bytes
        }
    }

    #[inline(always)]
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.bytes.is_empty() { None }
        else { Some(self.bytes.remove(0)) }
    }

    #[inline(always)]
    pub fn read<T>(&mut self) -> Option<T> where T: IOAble {
        T::read_from(self)
    }
}

pub struct OutputStream {
    pub bytes: Vec<u8>
}

impl OutputStream {
    pub fn new() -> Self {
        OutputStream {
            bytes: Vec::new()
        }
    }

    #[inline(always)]
    pub fn write_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    #[inline(always)]
    pub fn write<T>(&mut self, stuff: T) where T: IOAble {
        stuff.write_to(self);
    }
}

pub trait IOAble: Sized {
    // type Output = Self;
    fn read_from(stream: &mut InputStream) -> Option<Self>;
    fn write_to(&self, stream: &mut OutputStream);
}

impl IOAble for i8 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        stream.read_byte().map(|b| b as i8)
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte(*self as u8);
    }
}

impl IOAble for u8 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        stream.read_byte()
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte(*self);
    }
}

impl IOAble for i16 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(i16::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);
    }
}

impl IOAble for u16 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(u16::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);
    }
}

impl IOAble for i32 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(i32::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 24) as u8);
        stream.write_byte((self >> 16) as u8);
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);
    }
}

impl IOAble for u32 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(u32::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }
    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 24) as u8);
        stream.write_byte((self >> 16) as u8);
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);
    }
}

impl IOAble for i64 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(i64::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 56) as u8);
        stream.write_byte((self >> 48) as u8);
        stream.write_byte((self >> 40) as u8);
        stream.write_byte((self >> 32) as u8);
        stream.write_byte((self >> 24) as u8);
        stream.write_byte((self >> 16) as u8);
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);
    }
}

impl IOAble for u64 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(u64::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self >> 56) as u8);
        stream.write_byte((self >> 48) as u8);
        stream.write_byte((self >> 40) as u8);
        stream.write_byte((self >> 32) as u8);
        stream.write_byte((self >> 24) as u8);
        stream.write_byte((self >> 16) as u8);
        stream.write_byte((self >> 8) as u8);
        stream.write_byte(*self as u8);

    }
}

impl IOAble for f32 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(f32::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }
    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self.to_bits() >> 24) as u8);
        stream.write_byte((self.to_bits() >> 16) as u8);
        stream.write_byte((self.to_bits() >> 8) as u8);
        stream.write_byte(self.to_bits() as u8);
    }
}

impl IOAble for f64 {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(f64::from_be_bytes([
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?,
            stream.read_byte()?
        ]))
    }
    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte((self.to_bits() >> 56) as u8);
        stream.write_byte((self.to_bits() >> 48) as u8);
        stream.write_byte((self.to_bits() >> 40) as u8);
        stream.write_byte((self.to_bits() >> 32) as u8);
        stream.write_byte((self.to_bits() >> 24) as u8);
        stream.write_byte((self.to_bits() >> 16) as u8);
        stream.write_byte((self.to_bits() >> 8) as u8);
        stream.write_byte(self.to_bits() as u8);
    }
}

impl IOAble for &str {
    fn read_from(_: &mut InputStream) -> Option<Self> {
        None
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write(self.to_string());
    }
}

impl IOAble for String {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        let length: u32 = stream.read()?;
        let mut bytes = vec![0u8; length as usize];
        for item in bytes.iter_mut() {
            *item = stream.read_byte()?;
        }

        if let Ok(string) = String::from_utf8(bytes) {
            Some(string)
        }
        else {
            None
        }
    }
    fn write_to(&self, stream: &mut OutputStream) {
        let bytes = self.as_bytes();
        stream.write(bytes.len() as u32);
        for byte in bytes {
            stream.write_byte(*byte);
        }
    }
}

impl IOAble for bool {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        Some(stream.read_byte()? != 0)
    }
    fn write_to(&self, stream: &mut OutputStream) {
        stream.write_byte(if *self { 1 } else { 0 });
    }
}

impl<T> IOAble for Vec<T> where T: IOAble {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        let length: i32 = stream.read()?;
        let mut array = Vec::with_capacity(length as usize);
        for _ in 0..length {
            array.push(T::read_from(stream)?);
        }

        Some(array)
    }

    fn write_to(&self, stream: &mut OutputStream) {
        stream.write(self.len() as i32);
        for item in self {
            item.write_to(stream);
        }
    }
}

impl<T> IOAble for Option<T> where T: IOAble {
    fn read_from(stream: &mut InputStream) -> Option<Self> {
        if stream.read()? {
            Some(Some(T::read_from(stream)?))
        } else {
            Some(None)
        }
    }

    fn write_to(&self, stream: &mut OutputStream) {
        match self {
            Some(item) => {
                stream.write(true);
                item.write_to(stream);
            },
            None => {
                stream.write(false);
            }
        }
    }
}

impl<T> IOAble for &T where T: IOAble {
    fn read_from(_: &mut InputStream) -> Option<Self> {
        None
    }

    fn write_to(&self, stream: &mut OutputStream) {
        (**self).write_to(stream);
    }
}
