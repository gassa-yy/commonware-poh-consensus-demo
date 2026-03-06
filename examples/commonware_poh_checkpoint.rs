use commonware_poh_consensus_demo::commonware::checkpoint::{
    PohCheckpointPayload, MAX_CHECKPOINT_APP_BYTES,
};
use commonware_poh_consensus_demo::core::poh::Poh;

#[derive(Debug, Clone)]
struct Validator {
    id: &'static str,
    power: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validators = vec![
        Validator { id: "A", power: 5 },
        Validator { id: "B", power: 3 },
        Validator { id: "C", power: 2 },
    ];
    let total_power: u64 = validators.iter().map(|v| v.power).sum();
    let quorum = (total_power * 2 / 3) + 1;

    println!("commonware_poh_checkpoint_demo");
    println!("validators: {:?}", validators);
    println!("total_power={total_power}, quorum={quorum}");
    println!("mempool=disabled (checkpoint-only proposals)");

    let mut poh = Poh::new(2);
    let mut finalized = 0u64;

    for height in 1..=5u64 {
        for e in 0..2u64 {
            let event = format!("height={height};event={e}");
            poh.advance(&event);
        }

        if !poh.should_emit_checkpoint() {
            continue;
        }

        let checkpoint = poh.checkpoint();
        let app_data = format!("checkpoint@height={height}").into_bytes();
        let payload =
            PohCheckpointPayload::from_checkpoint(height, 0, &checkpoint, app_data)?;
        let encoded = payload.encode()?;
        let decoded = PohCheckpointPayload::decode(&encoded)?;

        let mut voted_power = 0u64;
        for validator in &validators {
            voted_power += validator.power;
            println!("vote: validator={} power={}", validator.id, validator.power);
            if voted_power >= quorum {
                break;
            }
        }

        if voted_power >= quorum {
            finalized += 1;
            println!(
                "finalized: height={} tick={} head={} encoded_bytes={} app_bytes={}",
                decoded.height,
                decoded.checkpoint_tick,
                decoded.checkpoint_head_hex,
                encoded.len(),
                decoded.app_data.len()
            );
        } else {
            println!("failed: height={height} (quorum not reached)");
        }
    }

    println!("summary: finalized_heights={finalized}/5");
    println!("payload_limit_bytes={MAX_CHECKPOINT_APP_BYTES}");
    Ok(())
}
