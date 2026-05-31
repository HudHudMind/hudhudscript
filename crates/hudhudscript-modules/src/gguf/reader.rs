//! GGUF binary reader — low-level primitive extraction.

use super::{GgufError, GgufValue};

pub struct Reader<'a> {
    data: &'a [u8],
    pub(crate) pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_u8(&mut self) -> Result<u8, GgufError> {
        if self.remaining() < 1 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_u32(&mut self) -> Result<u32, GgufError> {
        if self.remaining() < 4 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn read_u64(&mut self) -> Result<u64, GgufError> {
        if self.remaining() < 8 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = u64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    pub fn read_i32(&mut self) -> Result<i32, GgufError> {
        if self.remaining() < 4 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = i32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn read_i64(&mut self) -> Result<i64, GgufError> {
        if self.remaining() < 8 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = i64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    pub fn read_f32(&mut self) -> Result<f32, GgufError> {
        if self.remaining() < 4 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = f32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn read_f64(&mut self) -> Result<f64, GgufError> {
        if self.remaining() < 8 {
            return Err(GgufError::UnexpectedEof);
        }
        let v = f64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    pub fn read_bool(&mut self) -> Result<bool, GgufError> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_string(&mut self) -> Result<String, GgufError> {
        let len = self.read_u64()? as usize;
        if self.remaining() < len {
            return Err(GgufError::UnexpectedEof);
        }
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .map_err(|_| GgufError::InvalidUtf8)?;
        self.pos += len;
        Ok(s.to_string())
    }
}
