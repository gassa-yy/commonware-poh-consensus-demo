use commonware_poh_consensus_demo::commonware::checkpoint::{
    PayloadError, PohCheckpointPayload, CHECKPOINT_PAYLOAD_VERSION, MAX_CHECKPOINT_APP_BYTES,
};
use commonware_poh_consensus_demo::core::types::PohCheckpoint;

fn sample_checkpoint() -> PohCheckpoint {
    PohCheckpoint {
        tick: 77,
        head: "0123abc456def789".to_string(),
    }
}

#[test]
fn payload_roundtrip_encoding() {
    let payload = PohCheckpointPayload::from_checkpoint(
        7,
        3,
        &sample_checkpoint(),
        b"checkpoint-only".to_vec(),
    )
    .expect("valid payload");

    let encoded = payload.encode().expect("encode");
    let decoded = PohCheckpointPayload::decode(&encoded).expect("decode");
    assert_eq!(decoded, payload);
}

#[test]
fn payload_boundary_max_allowed_bytes() {
    let payload = PohCheckpointPayload::from_checkpoint(
        7,
        0,
        &sample_checkpoint(),
        vec![1u8; MAX_CHECKPOINT_APP_BYTES],
    )
    .expect("max payload should be accepted");
    let encoded = payload.encode().expect("encode");
    let decoded = PohCheckpointPayload::decode(&encoded).expect("decode");
    assert_eq!(decoded.app_data.len(), MAX_CHECKPOINT_APP_BYTES);
}

#[test]
fn payload_boundary_rejects_oversized_bytes() {
    let err = PohCheckpointPayload::from_checkpoint(
        7,
        0,
        &sample_checkpoint(),
        vec![1u8; MAX_CHECKPOINT_APP_BYTES + 1],
    )
    .expect_err("payload over boundary must fail");

    assert!(matches!(err, PayloadError::AppDataTooLarge { .. }));
}

#[test]
fn payload_decode_rejects_wrong_version() {
    let mut encoded = PohCheckpointPayload::from_checkpoint(
        7,
        1,
        &sample_checkpoint(),
        b"x".to_vec(),
    )
    .expect("valid payload")
    .encode()
    .expect("encode");
    encoded[0] = CHECKPOINT_PAYLOAD_VERSION.saturating_add(1);

    let err = PohCheckpointPayload::decode(&encoded).expect_err("wrong version must fail");
    assert!(matches!(err, PayloadError::InvalidVersion { .. }));
}

#[test]
fn payload_decode_rejects_truncated_buffer() {
    let encoded = PohCheckpointPayload::from_checkpoint(
        7,
        1,
        &sample_checkpoint(),
        b"x".to_vec(),
    )
    .expect("valid payload")
    .encode()
    .expect("encode");

    let truncated = &encoded[..encoded.len() - 1];
    let err = PohCheckpointPayload::decode(truncated).expect_err("truncated must fail");
    assert!(matches!(err, PayloadError::BufferTooShort));
}
