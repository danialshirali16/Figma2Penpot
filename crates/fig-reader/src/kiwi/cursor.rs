use std::fmt;

/// Bounds-checked reader for Kiwi's binary primitives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CursorError {
    Truncated(&'static str),
    Invalid(String),
}

impl fmt::Display for CursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated(what) => write!(f, "truncated while reading {what}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

pub(crate) type CursorResult<T> = Result<T, CursorError>;

#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub(crate) fn position(&self) -> usize {
        self.position
    }

    pub(crate) fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub(crate) fn read_u8(&mut self) -> CursorResult<u8> {
        let byte = *self
            .data
            .get(self.position)
            .ok_or(CursorError::Truncated("byte"))?;
        self.position += 1;
        Ok(byte)
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> CursorResult<&'a [u8]> {
        let end = self
            .position
            .checked_add(len)
            .ok_or_else(|| CursorError::Invalid("byte range overflow".into()))?;
        let bytes = self
            .data
            .get(self.position..end)
            .ok_or(CursorError::Truncated("byte array"))?;
        self.position = end;
        Ok(bytes)
    }

    pub(crate) fn skip(&mut self, len: usize) -> CursorResult<()> {
        let end = self
            .position
            .checked_add(len)
            .ok_or_else(|| CursorError::Invalid("byte range overflow".into()))?;
        if end > self.data.len() {
            return Err(CursorError::Truncated("skipped bytes"));
        }
        self.position = end;
        Ok(())
    }

    pub(crate) fn read_bool(&mut self) -> CursorResult<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(CursorError::Invalid(format!(
                "boolean byte must be 0 or 1, found {other}"
            ))),
        }
    }

    pub(crate) fn read_var_uint(&mut self) -> CursorResult<u32> {
        let mut result: u64 = 0;
        for index in 0..5 {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u64) << (index * 7);
            if byte & 0x80 == 0 {
                if result > u32::MAX as u64 {
                    return Err(CursorError::Invalid(
                        "variable-length unsigned integer exceeds u32".into(),
                    ));
                }
                return Ok(result as u32);
            }
        }
        Err(CursorError::Invalid(
            "variable-length unsigned integer is longer than 5 bytes".into(),
        ))
    }

    pub(crate) fn read_var_int(&mut self) -> CursorResult<i32> {
        let value = self.read_var_uint()? as i64;
        Ok(((value >> 1) as i32) ^ -((value & 1) as i32))
    }

    pub(crate) fn read_var_uint64(&mut self) -> CursorResult<u64> {
        let mut result: u128 = 0;
        for index in 0..10 {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u128) << (index * 7);
            if byte & 0x80 == 0 {
                if result > u64::MAX as u128 {
                    return Err(CursorError::Invalid(
                        "variable-length unsigned integer exceeds u64".into(),
                    ));
                }
                return Ok(result as u64);
            }
        }
        Err(CursorError::Invalid(
            "variable-length unsigned integer is longer than 10 bytes".into(),
        ))
    }

    pub(crate) fn read_var_int64(&mut self) -> CursorResult<i64> {
        let value = self.read_var_uint64()? as i128;
        Ok(((value >> 1) as i64) ^ -((value & 1) as i64))
    }

    /// Kiwi stores zero as one byte and other floats as a rotated 32-bit value.
    pub(crate) fn read_float(&mut self) -> CursorResult<f32> {
        let first = self.read_u8()?;
        if first == 0 {
            return Ok(0.0);
        }
        let rest = self.read_bytes(3)?;
        let bits = (first as u32)
            | ((rest[0] as u32) << 8)
            | ((rest[1] as u32) << 16)
            | ((rest[2] as u32) << 24);
        Ok(f32::from_bits(bits.rotate_left(23)))
    }

    /// NUL-terminated UTF-8 string, capped before allocation.
    pub(crate) fn read_string(&mut self, max_bytes: usize) -> CursorResult<&'a str> {
        let start = self.position;
        let rest = self
            .data
            .get(start..)
            .ok_or(CursorError::Truncated("string"))?;
        let end = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(CursorError::Truncated("unterminated string"))?;
        if end > max_bytes {
            return Err(CursorError::Invalid(format!(
                "string is {end} bytes, limit is {max_bytes}"
            )));
        }
        self.position = start + end + 1;
        let bytes = &rest[..end];
        std::str::from_utf8(bytes)
            .map_err(|_| CursorError::Invalid("string is not valid UTF-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_uints() {
        assert_eq!(Cursor::new(&[0]).read_var_uint().unwrap(), 0);
        assert_eq!(Cursor::new(&[1]).read_var_uint().unwrap(), 1);
        assert_eq!(Cursor::new(&[0xac, 0x02]).read_var_uint().unwrap(), 300);
        assert!(Cursor::new(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01])
            .read_var_uint()
            .is_err());
    }

    #[test]
    fn signed_ints_are_zigzag() {
        assert_eq!(Cursor::new(&[0]).read_var_int().unwrap(), 0);
        assert_eq!(Cursor::new(&[1]).read_var_int().unwrap(), -1);
        assert_eq!(Cursor::new(&[2]).read_var_int().unwrap(), 1);
        assert_eq!(Cursor::new(&[3]).read_var_int().unwrap(), -2);
    }

    #[test]
    fn floats_roundtrip_zero_only() {
        assert_eq!(Cursor::new(&[0]).read_float().unwrap(), 0.0);
        assert!(Cursor::new(&[1, 0, 0]).read_float().is_err());
    }

    #[test]
    fn strings_are_bounded() {
        assert_eq!(Cursor::new(b"hi\0").read_string(4).unwrap(), "hi");
        assert!(Cursor::new(b"hi\0").read_string(1).is_err());
        assert!(Cursor::new(b"hi").read_string(4).is_err());
        assert!(Cursor::new(&[0xff, 0]).read_string(4).is_err());
    }
}
