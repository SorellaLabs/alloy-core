use crate::Signed;
use alloy_rlp::{
    length_of_length, BufMut, Decodable, DecodeError, Encodable, Header, EMPTY_STRING_CODE,
};

const MAX_BITS: usize = 55 * 8;

impl<const BITS: usize, const LIMBS: usize> Encodable for Signed<BITS, LIMBS> {
    #[inline]
    fn length(&self) -> usize {
        self.0.length()
    }

    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        self.0.encode(out)
    }
}

impl<const BITS: usize, const LIMBS: usize> Decodable for Signed<BITS, LIMBS> {
    #[inline]
    fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        //TODO: decode as unsigned
        //convert to signed type
    
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

    #[test]
    pub fn rlp_small() {
        let signed_int: Signed<64, 1> = Signed::from_dec_str("-1260332").unwrap();

        let mut buf = BytesMut::new();
        signed_int.encode(&mut buf);
        let freezed = buf.freeze();
        let mut rlp = freezed.as_ref();
        let result: Signed<64, 1> = Signed::decode(&mut rlp).unwrap();

        assert_eq!(result, signed_int);
    }

    #[test]
    pub fn rlp_big() {
        let signed_int: Signed<256, 4> =
            Signed::from_dec_str("-1260332358234975439857342953478925").unwrap();

        let mut buf = BytesMut::new();
        signed_int.encode(&mut buf);
        let freezed = buf.freeze();
        let mut rlp = freezed.as_ref();
        let result: Signed<256, 4> = Signed::decode(&mut rlp).unwrap();

        assert_eq!(result, signed_int);
    }
}
