use super::*;

// TODO: This should probably be in U/D-Info
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DtmfKind {
    /// ETSI EN 300 392-2 V3.x: DTMF type = 000 (digits present)
    ToneStart,
    /// ETSI EN 300 392-2 V3.x: DTMF type = 001
    ToneEnd,
    /// ETSI EN 300 392-2 V3.x: DTMF type = 010
    NotSupported,
    /// ETSI EN 300 392-2 V3.x: DTMF type = 011
    NotSubscribed,
    /// ETSI EN 300 392-2 V3.x: reserved values 100..111
    Reserved(u8),
    /// Legacy edition-1 style payload (length divisible by 4): digits only, no 3-bit type.
    LegacyDigits,
    /// Payload could not be interpreted according to either format.
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DtmfDecoded {
    pub(super) kind: DtmfKind,
    pub(super) digits: String,
    pub(super) parsed_bits: usize,
    pub(super) full_len_bits: usize,
    pub(super) malformed: bool,
}

#[inline]
fn decode_dtmf_digit(nibble: u8) -> Option<char> {
    match nibble {
        0..=9 => Some(char::from(b'0' + nibble)),
        0x0a => Some('*'),
        0x0b => Some('#'),
        0x0c => Some('A'),
        0x0d => Some('B'),
        0x0e => Some('C'),
        0x0f => Some('D'),
        _ => None,
    }
}

pub(super) fn decode_dtmf(field: &Type3FieldGeneric) -> DtmfDecoded {
    let full_len_bits = field.len;
    let len_bits = full_len_bits.min(64);
    if len_bits == 0 {
        return DtmfDecoded {
            kind: DtmfKind::Invalid,
            digits: String::new(),
            parsed_bits: 0,
            full_len_bits,
            malformed: true,
        };
    }

    // Legacy mechanism (edition-1): payload is 4-bit digit nibbles only.
    // ETSI EN 300 392-2 V3.x note: new mechanism length is not divisible by 4.
    if len_bits % 4 == 0 {
        let nibble_count = len_bits / 4;
        let mut digits = String::with_capacity(nibble_count);
        for i in 0..nibble_count {
            let shift = (nibble_count - 1 - i) * 4;
            let nibble = ((field.data >> shift) & 0x0f) as u8;
            if let Some(c) = decode_dtmf_digit(nibble) {
                digits.push(c);
            }
        }
        return DtmfDecoded {
            kind: DtmfKind::LegacyDigits,
            digits,
            parsed_bits: len_bits,
            full_len_bits,
            malformed: false,
        };
    }

    if len_bits < 3 {
        return DtmfDecoded {
            kind: DtmfKind::Invalid,
            digits: String::new(),
            parsed_bits: len_bits,
            full_len_bits,
            malformed: true,
        };
    }

    let dtmf_type = ((field.data >> (len_bits - 3)) & 0x07) as u8;
    let tail_bits = len_bits - 3;

    let mut digits = String::new();
    let mut malformed = false;
    let kind = match dtmf_type {
        0 => {
            if tail_bits == 0 || tail_bits % 4 != 0 {
                malformed = true;
            } else {
                let nibble_count = tail_bits / 4;
                digits.reserve(nibble_count);
                for i in 0..nibble_count {
                    let shift = tail_bits - 4 * (i + 1);
                    let nibble = ((field.data >> shift) & 0x0f) as u8;
                    if let Some(c) = decode_dtmf_digit(nibble) {
                        digits.push(c);
                    }
                }
            }
            DtmfKind::ToneStart
        }
        1 => {
            if tail_bits != 0 {
                malformed = true;
            }
            DtmfKind::ToneEnd
        }
        2 => {
            if tail_bits != 0 {
                malformed = true;
            }
            DtmfKind::NotSupported
        }
        3 => {
            if tail_bits != 0 {
                malformed = true;
            }
            DtmfKind::NotSubscribed
        }
        4..=7 => {
            if tail_bits != 0 {
                malformed = true;
            }
            DtmfKind::Reserved(dtmf_type)
        }
        _ => DtmfKind::Invalid,
    };

    DtmfDecoded {
        kind,
        digits,
        parsed_bits: len_bits,
        full_len_bits,
        malformed,
    }
}

pub(super) fn pack_type3_bits_to_bytes(field: &Type3FieldGeneric) -> (u16, Vec<u8>) {
    // Type3FieldGeneric stores up to 64 bits of payload; longer fields are already truncated on parse.
    let len_bits = field.len.min(64);
    if len_bits == 0 {
        return (0, Vec::new());
    }

    let mut out = vec![0u8; len_bits.div_ceil(8)];
    let src = field.data;
    for bit_idx in 0..len_bits {
        let src_shift = len_bits - 1 - bit_idx;
        let bit = ((src >> src_shift) & 0x1) as u8;
        let byte_idx = bit_idx / 8;
        let bit_pos = 7 - (bit_idx % 8);
        out[byte_idx] |= bit << bit_pos;
    }
    (len_bits as u16, out)
}
