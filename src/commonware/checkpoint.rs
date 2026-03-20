use crate::core::types::PohCheckpoint;

pub const CHECKPOINT_PAYLOAD_VERSION: u8 = 1;
pub const MAX_CHECKPOINT_APP_BYTES: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PohCheckpointPayload {
    pub height: u64,
    pub round: u64,
    pub checkpoint_tick: u64,
    pub checkpoint_head_hex: String,
    pub app_data: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PayloadError {
    AppDataTooLarge { got: usize, max: usize },
    CheckpointHeadNotHex,
    CheckpointHeadWrongLength { got: usize, expected: usize },
    BufferTooShort,
    InvalidVersion { got: u8, expected: u8 },
    TrailingBytes,
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadError::AppDataTooLarge { got, max } => {
                write!(f, "app_data exceeds max size: got {got}, max {max}")
            }
            PayloadError::CheckpointHeadNotHex => f.write_str("checkpoint head is not valid hex"),
            PayloadError::CheckpointHeadWrongLength { got, expected } => write!(
                f,
                "checkpoint head must have {expected} hex chars, got {got}"
            ),
            PayloadError::BufferTooShort => f.write_str("encoded payload is too short"),
            PayloadError::InvalidVersion { got, expected } => {
                write!(
                    f,
                    "invalid payload version: got {got}, expected {expected}"
                )
            }
            PayloadError::TrailingBytes => f.write_str("encoded payload has trailing bytes"),
        }
    }
}

impl std::error::Error for PayloadError {}

impl PohCheckpointPayload {
    pub fn from_checkpoint(
        height: u64,
        round: u64,
        checkpoint: &PohCheckpoint,
        app_data: Vec<u8>,
    ) -> Result<Self, PayloadError> {
        let payload = Self {
            height,
            round,
            checkpoint_tick: checkpoint.tick,
            checkpoint_head_hex: checkpoint.head.clone(),
            app_data,
        };
        payload.validate()?;
        Ok(payload)
    }

    pub fn validate(&self) -> Result<(), PayloadError> {
        if self.app_data.len() > MAX_CHECKPOINT_APP_BYTES {
            return Err(PayloadError::AppDataTooLarge {
                got: self.app_data.len(),
                max: MAX_CHECKPOINT_APP_BYTES,
            });
        }

        let expected_hex_len = 16usize;
        if self.checkpoint_head_hex.len() != expected_hex_len {
            return Err(PayloadError::CheckpointHeadWrongLength {
                got: self.checkpoint_head_hex.len(),
                expected: expected_hex_len,
            });
        }

        if !self
            .checkpoint_head_hex
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        {
            return Err(PayloadError::CheckpointHeadNotHex);
        }

        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, PayloadError> {
        self.validate()?;
        let mut out = Vec::with_capacity(1 + 8 + 8 + 8 + 8 + 2 + self.app_data.len());
        out.push(CHECKPOINT_PAYLOAD_VERSION);
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.round.to_le_bytes());
        out.extend_from_slice(&self.checkpoint_tick.to_le_bytes());

        let mut head = [0u8; 8];
        decode_hex_16_to_8(&self.checkpoint_head_hex, &mut head)?;
        out.extend_from_slice(&head);

        let app_len = self.app_data.len() as u16;
        out.extend_from_slice(&app_len.to_le_bytes());
        out.extend_from_slice(&self.app_data);
        Ok(out)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, PayloadError> {
        const BASE_LEN: usize = 1 + 8 + 8 + 8 + 8 + 2;
        if buf.len() < BASE_LEN {
            return Err(PayloadError::BufferTooShort);
        }

        let mut idx = 0usize;
        let version = buf[idx];
        idx += 1;
        if version != CHECKPOINT_PAYLOAD_VERSION {
            return Err(PayloadError::InvalidVersion {
                got: version,
                expected: CHECKPOINT_PAYLOAD_VERSION,
            });
        }

        let height = read_u64(buf, &mut idx)?;
        let round = read_u64(buf, &mut idx)?;
        let checkpoint_tick = read_u64(buf, &mut idx)?;

        let head = read_arr_8(buf, &mut idx)?;
        let checkpoint_head_hex = encode_8_as_hex_16(&head);

        let app_len = read_u16(buf, &mut idx)? as usize;
        let remaining = buf.len().saturating_sub(idx);
        if remaining < app_len {
            return Err(PayloadError::BufferTooShort);
        }

        let app_data = buf[idx..idx + app_len].to_vec();
        idx += app_len;
        if idx != buf.len() {
            return Err(PayloadError::TrailingBytes);
        }

        let payload = Self {
            height,
            round,
            checkpoint_tick,
            checkpoint_head_hex,
            app_data,
        };
        payload.validate()?;
        Ok(payload)
    }
}

fn read_u64(buf: &[u8], idx: &mut usize) -> Result<u64, PayloadError> {
    if buf.len().saturating_sub(*idx) < 8 {
        return Err(PayloadError::BufferTooShort);
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[*idx..*idx + 8]);
    *idx += 8;
    Ok(u64::from_le_bytes(bytes))
}

fn read_u16(buf: &[u8], idx: &mut usize) -> Result<u16, PayloadError> {
    if buf.len().saturating_sub(*idx) < 2 {
        return Err(PayloadError::BufferTooShort);
    }
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&buf[*idx..*idx + 2]);
    *idx += 2;
    Ok(u16::from_le_bytes(bytes))
}

fn read_arr_8(buf: &[u8], idx: &mut usize) -> Result<[u8; 8], PayloadError> {
    if buf.len().saturating_sub(*idx) < 8 {
        return Err(PayloadError::BufferTooShort);
    }
    let mut out = [0u8; 8];
    out.copy_from_slice(&buf[*idx..*idx + 8]);
    *idx += 8;
    Ok(out)
}

fn decode_hex_16_to_8(hex: &str, out: &mut [u8; 8]) -> Result<(), PayloadError> {
    if hex.len() != 16 {
        return Err(PayloadError::CheckpointHeadWrongLength {
            got: hex.len(),
            expected: 16,
        });
    }

    let bytes = hex.as_bytes();
    for i in 0..8usize {
        let hi = decode_nibble(bytes[i * 2]).ok_or(PayloadError::CheckpointHeadNotHex)?;
        let lo = decode_nibble(bytes[i * 2 + 1]).ok_or(PayloadError::CheckpointHeadNotHex)?;
        out[i] = (hi << 4) | lo;
    }
    Ok(())
}

fn decode_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

fn encode_8_as_hex_16(bytes: &[u8; 8]) -> String {
    let mut out = String::with_capacity(16);
    for b in bytes {
        out.push(nibble_to_hex(b >> 4));
        out.push(nibble_to_hex(b & 0x0f));
    }
    out
}

fn nibble_to_hex(v: u8) -> char {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    HEX[v as usize] as char
}

#[cfg(test)]
mod tests {
    use super::{PayloadError, PohCheckpointPayload, MAX_CHECKPOINT_APP_BYTES};
    use crate::core::types::PohCheckpoint;

    #[test]
    fn roundtrip_encode_decode() {
        let payload = PohCheckpointPayload::from_checkpoint(
            42,
            1,
            &PohCheckpoint {
                tick: 9,
                head: "0123abc456def789".to_string(),
            },
            b"abc".to_vec(),
        )
        .expect("valid payload");

        let encoded = payload.encode().expect("encode");
        let decoded = PohCheckpointPayload::decode(&encoded).expect("decode");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn rejects_oversized_payload() {
        let err = PohCheckpointPayload::from_checkpoint(
            1,
            0,
            &PohCheckpoint {
                tick: 1,
                head: "0123abc456def789".to_string(),
            },
            vec![0u8; MAX_CHECKPOINT_APP_BYTES + 1],
        )
        .expect_err("must reject oversized app_data");
        assert!(matches!(err, PayloadError::AppDataTooLarge { .. }));
    }
}
