use commonware_consensus::types::{Epoch, Round, View};
use commonware_cryptography::{ed25519, Hasher, Sha256, Signer, Verifier};
use commonware_p2p::Recipients;

use crate::commonware::checkpoint::PohCheckpointPayload;
use crate::core::poh::Poh;

const NS_VOTE: &[u8] = b"poh/checkpoint/vote/v1";

#[derive(Clone)]
pub struct DemoValidator {
    pub id: &'static str,
    pub power: u64,
    pub key: ed25519::PrivateKey,
}

#[derive(Debug, Clone)]
pub struct FinalizationRecord {
    pub height: u64,
    pub round: Round,
    pub tick: u64,
    pub digest_hex: String,
    pub voted_power: u64,
    pub quorum: u64,
}

#[derive(Debug, Clone)]
pub struct DemoSummary {
    pub finalized: Vec<FinalizationRecord>,
    pub quorum: u64,
    pub validator_count: usize,
}

fn make_validators() -> Vec<DemoValidator> {
    vec![
        DemoValidator {
            id: "A",
            power: 5,
            key: ed25519::PrivateKey::from_seed(11),
        },
        DemoValidator {
            id: "B",
            power: 3,
            key: ed25519::PrivateKey::from_seed(22),
        },
        DemoValidator {
            id: "C",
            power: 2,
            key: ed25519::PrivateKey::from_seed(33),
        },
    ]
}

pub fn run_checkpoint_consensus_demo(heights: u64) -> Result<DemoSummary, String> {
    let validators = make_validators();
    let total_power: u64 = validators.iter().map(|v| v.power).sum();
    let quorum = (2 * total_power / 3) + 1;

    let mut poh = Poh::new(2);
    let mut finalized = Vec::new();

    for h in 1..=heights {
        poh.advance(&format!("h={h};e=0"));
        poh.advance(&format!("h={h};e=1"));
        if !poh.should_emit_checkpoint() {
            continue;
        }

        let checkpoint = poh.checkpoint();
        let payload = PohCheckpointPayload::from_checkpoint(
            h,
            0,
            &checkpoint,
            format!("checkpoint@{h}").into_bytes(),
        )
        .map_err(|e| e.to_string())?;
        let encoded = payload.encode().map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        let digest = hasher.finalize();

        // Sign + verify votes until quorum is reached.
        let mut voted_power = 0u64;
        for v in &validators {
            let sig = v.key.sign(NS_VOTE, digest.as_ref());
            let ok = v.key.public_key().verify(NS_VOTE, digest.as_ref(), &sig);
            if !ok {
                return Err(format!("signature verify failed for validator {}", v.id));
            }
            voted_power += v.power;
            if voted_power >= quorum {
                break;
            }
        }

        if voted_power < quorum {
            return Err(format!("quorum not reached at height {h}"));
        }

        let recipients = Recipients::Some(validators.iter().map(|v| v.key.public_key()).collect());
        let _recipient_count = match recipients {
            Recipients::All => 0,
            Recipients::One(_) => 1,
            Recipients::Some(ref v) => v.len(),
        };

        finalized.push(FinalizationRecord {
            height: h,
            round: Round::new(Epoch::new(0), View::new(1)),
            tick: checkpoint.tick,
            digest_hex: format!("{digest}"),
            voted_power,
            quorum,
        });
    }

    Ok(DemoSummary {
        finalized,
        quorum,
        validator_count: validators.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::run_checkpoint_consensus_demo;

    #[test]
    fn finalizes_all_heights() {
        let res = run_checkpoint_consensus_demo(5).expect("demo must run");
        assert_eq!(res.finalized.len(), 5);
        for rec in &res.finalized {
            assert!(rec.voted_power >= rec.quorum);
        }
    }
}
