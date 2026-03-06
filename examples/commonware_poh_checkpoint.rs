use commonware_consensus::{Automaton, CertifiableAutomaton, Relay, Reporter};
use commonware_cryptography::{ed25519, Hasher, Sha256, Signer};
use commonware_p2p::Recipients;
use commonware_poh_consensus_demo::commonware::checkpoint::PohCheckpointPayload;
use commonware_poh_consensus_demo::core::poh::Poh;
use commonware_poh_consensus_demo::core::types::PohCheckpoint;
use commonware_utils::channel::oneshot;

#[derive(Clone, Debug)]
struct DemoContext {
    height: u64,
    round: u64,
}

#[derive(Clone)]
struct DemoAutomaton {
    checkpoint: PohCheckpoint,
}

impl Automaton for DemoAutomaton {
    type Context = DemoContext;
    type Digest = commonware_cryptography::sha256::Digest;

    async fn genesis(&mut self, _epoch: commonware_consensus::types::Epoch) -> Self::Digest {
        let mut h = Sha256::new();
        h.update(b"genesis");
        h.finalize()
    }

    async fn propose(&mut self, context: Self::Context) -> oneshot::Receiver<Self::Digest> {
        let (tx, rx) = oneshot::channel();
        let payload = PohCheckpointPayload::from_checkpoint(
            context.height,
            context.round,
            &self.checkpoint,
            b"poh-checkpoint".to_vec(),
        )
        .expect("valid payload")
        .encode()
        .expect("encode");

        let mut h = Sha256::new();
        h.update(&payload);
        let digest = h.finalize();
        let _ = tx.send(digest);
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

impl CertifiableAutomaton for DemoAutomaton {}

#[derive(Clone)]
struct DemoRelay;
impl Relay for DemoRelay {
    type Digest = commonware_cryptography::sha256::Digest;

    async fn broadcast(&mut self, payload: Self::Digest) {
        println!("relay.broadcast digest={payload}");
    }
}

#[derive(Clone)]
struct DemoReporter;
impl Reporter for DemoReporter {
    type Activity = String;

    async fn report(&mut self, activity: Self::Activity) {
        println!("reporter.activity={activity}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Commonware interface demo: PoH checkpoint consensus (no mempool)");
    println!(
        "runtime module: {}",
        std::any::type_name::<commonware_runtime::deterministic::Config>()
    );

    let mut poh = Poh::new(2);
    poh.advance("event-a");
    poh.advance("event-b");
    let checkpoint = poh.checkpoint();

    let mut automaton = DemoAutomaton {
        checkpoint: checkpoint.clone(),
    };
    let mut relay = DemoRelay;
    let mut reporter = DemoReporter;

    let _genesis = automaton.genesis(commonware_consensus::types::Epoch::new(0)).await;
    let digest = automaton
        .propose(DemoContext {
            height: 1,
            round: 0,
        })
        .await
        .await?;

    let verified = automaton
        .verify(
            DemoContext {
                height: 1,
                round: 0,
            },
            digest,
        )
        .await
        .await?;

    let certified = automaton
         .certify(commonware_consensus::types::Round::new(commonware_consensus::types::Epoch::new(0), commonware_consensus::types::View::new(1)), digest)
        .await
        .await
        .unwrap_or(false);

    relay.broadcast(digest).await;
    reporter
        .report(format!("height=1 round=0 verified={verified} certified={certified}"))
        .await;

    let pk = ed25519::PrivateKey::from_seed(7).public_key();
    let recipients = Recipients::One(pk);
    match recipients {
        Recipients::One(_) => println!("p2p recipients path: one peer"),
        Recipients::All => println!("p2p recipients path: all peers"),
        Recipients::Some(v) => println!("p2p recipients path: {} peers", v.len()),
    }

    println!("checkpoint tick={} head={}", checkpoint.tick, checkpoint.head);
    println!("done");
    Ok(())
}
