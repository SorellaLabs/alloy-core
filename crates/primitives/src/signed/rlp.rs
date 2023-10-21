use crate::Signed;
use alloy_rlp::{
    length_of_length, BufMut, Decodable, DecodeError, Encodable, Header, EMPTY_STRING_CODE,
};

const MAX_BITS: usize = 55 * 8;

impl<const BITS: usize, const LIMBS: usize> Encodable for Signed<BITS, LIMBS> {
    #[inline]
    fn length(&self) -> usize {
        let bits = self.0.bit_len();
        if bits <= 7 {
            1
        } else {
            let bytes = (bits + 7) / 8;
            bytes + length_of_length(bytes)
        }
    }

    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        // fast paths, avoiding allocation due to `to_be_bytes_vec`
        match LIMBS {
            0 => return out.put_u8(EMPTY_STRING_CODE),
            1 => return self.0.as_limbs()[0].encode(out),
            #[allow(clippy::cast_lossless)]
            2 => {
                return (self.0.as_limbs()[0] as u128 | ((self.0.as_limbs()[1] as u128) << 64))
                    .encode(out)
            }
            _ => {}
        }

        match self.0.bit_len() {
            0 => out.put_u8(EMPTY_STRING_CODE),
            1..=7 => {
                #[allow(clippy::cast_possible_truncation)] // self < 128
                out.put_u8(self.0.as_limbs()[0] as u8);
            }
            bits => {
                // avoid heap allocation in `to_be_bytes_vec`
                // SAFETY: we don't re-use `copy`
                #[cfg(target_endian = "little")]
                let mut copy = *&self.0;
                #[cfg(target_endian = "little")]
                let bytes = unsafe { copy.as_le_slice_mut() };
                #[cfg(target_endian = "little")]
                bytes.reverse();

                #[cfg(target_endian = "big")]
                let bytes = self.0.to_be_bytes_vec();

                let leading_zero_bytes = Self::BYTES - (bits + 7) / 8;
                let trimmed = &bytes[leading_zero_bytes..];
                if bits > MAX_BITS {
                    trimmed.encode(out);
                } else {
                    #[allow(clippy::cast_possible_truncation)] // bytes.len() < 56 < 256
                    out.put_u8(EMPTY_STRING_CODE + trimmed.len() as u8);
                    out.put_slice(trimmed);
                }
            }
        }
    }
}

impl<const BITS: usize, const LIMBS: usize> Decodable for Signed<BITS, LIMBS> {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let header = Header::decode(buf)?;
        if header.list {
            return Err(DecodeError::UnexpectedList)
        }
        let bytes = &buf[..header.payload_length];
        *buf = &buf[header.payload_length..];
        Self::try_from_be_slice(bytes).ok_or(DecodeError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    pub fn test_rlp() {
        let signed_int: Signed<64, 1> = Signed::from_dec_str("-1260332").unwrap();

        let mut buf = BytesMut::new();
        signed_int.encode(&mut buf);
        let freezed = buf.freeze();
        let mut rlp = freezed.as_ref();
        let result: Signed<64, 1> = Signed::decode(&mut rlp).unwrap();

        assert_eq!(result, signed_int);
    }
}
