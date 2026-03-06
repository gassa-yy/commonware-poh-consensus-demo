use commonware_consensus::types::{Epoch, Round, View};
use commonware_consensus::{Automaton, CertifiableAutomaton, Relay, Reporter};
use commonware_cryptography::{Hasher, Sha256};
use commonware_poh_consensus_demo::commonware::checkpoint::PohCheckpointPayload;
use commonware_poh_consensus_demo::commonware::demo::run_checkpoint_consensus_demo;
use commonware_poh_consensus_demo::core::poh::Poh;
use commonware_utils::channel::oneshot;

#[derive(Clone, Debug)]
struct Context {
    height: u64,
    round: u64,
}

#[derive(Clone)]
struct AdapterAutomaton {
    checkpoint_payload: Vec<u8>,
}

impl Automaton for AdapterAutomaton {
    type Context = Context;
    type Digest = commonware_cryptography::sha256::Digest;

    async fn genesis(&mut self, _epoch: Epoch) -> Self::Digest {
        let mut h = Sha256::new();
        h.update(b"genesis");
        h.finalize()
    }

    async fn propose(&mut self, context: Self::Context) -> oneshot::Receiver<Self::Digest> {
        let (tx, rx) = oneshot::channel();
        let mut h = Sha256::new();
        h.update(&self.checkpoint_payload);
        h.update(&context.height.to_le_bytes());
        h.update(&context.round.to_le_bytes());
        let _ = tx.send(h.finalize());
        rx
    }

    async fn verify(
        &mut self,
        _context: Self::Context,
        _payload: Self::Digest,
    ) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(true);
        rx
    }
}

impl CertifiableAutomaton for AdapterAutomaton {}

#[derive(Clone)]
struct AdapterRelay;
impl Relay for AdapterRelay {
    type Digest = commonware_cryptography::sha256::Digest;

    async fn broadcast(&mut self, payload: Self::Digest) {
        println!("relay.broadcast digest={payload}");
    }
}

#[derive(Clone)]
struct AdapterReporter;
impl Reporter for AdapterReporter {
    type Activity = String;

    async fn report(&mut self, activity: Self::Activity) {
        println!("reporter.activity={activity}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Commonware PoH checkpoint consensus demo (no mempool)");
    println!(
        "runtime module available: {}",
        std::any::type_name::<commonware_runtime::deterministic::Config>()
    );

    // Step 1: Build a PoH checkpoint payload that can be proposed by consensus.
    let mut poh = Poh::new(2);
    poh.advance("boot-event-1");
    poh.advance("boot-event-2");
    let checkpoint = poh.checkpoint();
    let payload = PohCheckpointPayload::from_checkpoint(1, 0, &checkpoint, b"bootstrap".to_vec())?;
    let encoded_payload = payload.encode()?;

    // Step 2: Drive the Commonware consensus trait boundary over the payload.
    let mut automaton = AdapterAutomaton {
        checkpoint_payload: encoded_payload,
    };
    let mut relay = AdapterRelay;
    let mut reporter = AdapterReporter;

    let _genesis = automaton.genesis(Epoch::new(0)).await;
    let digest = automaton
        .propose(Context {
            height: 1,
            round: 0,
        })
        .await
        .await?;

    let verified = automaton
        .verify(
            Context {
                height: 1,
                round: 0,
            },
            digest,
        )
        .await
        .await?;
    let certified = automaton
        .certify(Round::new(Epoch::new(0), View::new(1)), digest)
        .await
        .await
        .unwrap_or(false);

    relay.broadcast(digest).await;
    reporter
        .report(format!(
            "trait-boundary: verified={verified}, certified={certified}"
        ))
        .await;

    // Step 3: Run a full checkpoint-only finalization demo with signatures + quorum.
    let summary = run_checkpoint_consensus_demo(5)?;
    println!(
        "finalized_heights={}/5 validators={} quorum={}",
        summary.finalized.len(),
        summary.validator_count,
        summary.quorum
    );
    for rec in summary.finalized {
        println!(
            "finalized: h={} round=({},{}) tick={} voted_power={} digest={}",
            rec.height,
            rec.round.epoch().get(),
            rec.round.view().get(),
            rec.tick,
            rec.voted_power,
            rec.digest_hex
        );
    }

    Ok(())
}
