use crate::{Signed, Uint};
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
        let bytes = Uint::<BITS, LIMBS>::decode(buf)?;
        Ok(Self::from_raw(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    use crate::{
        aliases::{I0, I256, I64, I8, U0, U256, U64, U8},
        const_for, nlimbs,
    };
    use hex_literal::hex;
    use proptest::proptest;

    fn encode<T: Encodable>(value: T) -> Vec<u8> {
        let mut buf = vec![];
        value.encode(&mut buf);
        buf
    }

    #[test]
    fn test_rlp() {
        // See <https://github.com/paritytech/parity-common/blob/436cb0827f0e3238ccb80d7d453f756d126c0615/rlp/tests/tests.rs#L214>
        assert_eq!(encode(U0::from(0))[..], hex!("80"));
        assert_eq!(encode(U256::from(0))[..], hex!("80"));
        assert_eq!(encode(U256::from(15))[..], hex!("0f"));
        assert_eq!(encode(U256::from(1024))[..], hex!("820400"));
        assert_eq!(encode(U256::from(0x1234_5678))[..], hex!("8412345678"));
    }

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
