import hashlib
import random
from dataclasses import dataclass, field
from typing import Dict, List, Tuple

SEED = 42
random.seed(SEED)


def h(x: bytes) -> str:
    return hashlib.sha256(x).hexdigest()


@dataclass
class PoH:
    state: str = field(default_factory=lambda: h(b"genesis"))
    tick: int = 0
    checkpoint_every: int = 4

    def advance(self, payload: str) -> str:
        self.tick += 1
        self.state = h((self.state + f"|{self.tick}|{payload}").encode())
        return self.state

    def checkpoint(self) -> Tuple[int, str]:
        return self.tick, self.state


@dataclass
class Validator:
    vid: str
    power: int
    priority: int = 0


class WeightedRoundRobin:
    def __init__(self, vals: List[Validator]):
        self.vals = vals
        self.total = sum(v.power for v in vals)

    def select(self) -> Validator:
        for v in self.vals:
            v.priority += v.power
        chosen = max(self.vals, key=lambda x: (x.priority, x.vid))
        chosen.priority -= self.total
        return chosen


class BFT3Phase:
    def __init__(self, vals: List[Validator]):
        self.vals = vals
        self.total_power = sum(v.power for v in vals)
        self.quorum = (2 * self.total_power) // 3 + 1

    def run_round(self, height: int, round_id: int, proposer: Validator, poh_cp: Tuple[int, str]) -> bool:
        tick, cp = poh_cp
        block_id = f"H{height}-R{round_id}-P{proposer.vid}-T{tick}-{cp[:8]}"

        # Phase 1: Pre-Prepare
        # proposer broadcasts proposal with PoH checkpoint

        # Phase 2: Prepare
        prepare_power = 0
        for v in self.vals:
            # toy rule: 90% chance validator accepts valid-looking proposal
            ok = random.random() < 0.9
            if ok:
                prepare_power += v.power

        if prepare_power < self.quorum:
            print(f"[H{height} R{round_id}] PREPARE FAIL {prepare_power}/{self.total_power} < quorum {self.quorum}")
            return False

        # Phase 3: Commit
        commit_power = 0
        for v in self.vals:
            # toy rule: if prepare passed, commit likely follows
            ok = random.random() < 0.95
            if ok:
                commit_power += v.power

        if commit_power >= self.quorum:
            print(f"[H{height} R{round_id}] FINALIZED block={block_id} commit={commit_power}/{self.total_power}")
            return True

        print(f"[H{height} R{round_id}] COMMIT FAIL {commit_power}/{self.total_power} < quorum {self.quorum}")
        return False


def main():
    vals = [
        Validator("A", 5),
        Validator("B", 3),
        Validator("C", 2),
    ]

    rr = WeightedRoundRobin(vals)
    bft = BFT3Phase(vals)
    poh = PoH()

    max_heights = 6
    max_rounds = 5

    print("=== PoH + weighted round-robin + 3-phase BFT demo ===")
    for height in range(1, max_heights + 1):
        finalized = False
        for round_id in range(max_rounds):
            proposer = rr.select()

            # advance PoH chain with synthetic mempool/order events
            for i in range(random.randint(1, 3)):
                poh.advance(f"H{height}R{round_id}event{i}")

            cp = poh.checkpoint()
            print(f"[H{height} R{round_id}] proposer={proposer.vid} power={proposer.power} poh_tick={cp[0]} poh_head={cp[1][:10]}...")

            finalized = bft.run_round(height, round_id, proposer, cp)
            if finalized:
                break

        if not finalized:
            print(f"[H{height}] not finalized within {max_rounds} rounds (toy failure case)")


if __name__ == "__main__":
    main()
