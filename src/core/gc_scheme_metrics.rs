use crate::bag::GateCount;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metrics {
    communication: usize, // the data that operator send to each committe member in bytes
    computation: String,
    proving_cost: String,
    verifying_cost: String,
    assert_size: usize, // assert script size in bytes, assuming a proof of 128 bytes and a 20-byte public input
    disprove_size: usize, // disprove script size in bytes
}

pub trait GCScheme {
    fn core_metrics(gate_count: &GateCount) -> Metrics;
}

pub struct GRR2withZK;

impl GCScheme for GRR2withZK {
    fn core_metrics(gate_count: &GateCount) -> Metrics {
        let total_gate_count = gate_count.and
            + gate_count.or
            + gate_count.xor
            + gate_count.nand
            + gate_count.not
            + gate_count.xnor
            + gate_count.nimp
            + gate_count.nsor;
        let xor_gate_count = gate_count.xor + gate_count.xnor;
        let non_xor_gate_count = total_gate_count - xor_gate_count;

        // assuming we use 16 byte-hash
        let hash_size = 16;
        Metrics {
            communication: 2 * hash_size * non_xor_gate_count, // 2 hashes for each non-xor gate
            computation: format!("{}H, H is hash operation", 3 * non_xor_gate_count), // 3 hash operation for each non-xor gate
            proving_cost: format!("{}H, H is hash operation", 3 * non_xor_gate_count), // 3 hash operation needed to be prove for each non-xor gate,
            verifying_cost: format!("verify zk (almost none)"),
            assert_size: 32 * 148 * 8, // use sha256(32byte witness) as commitment scheme
            disprove_size: 32,         // challenger only need to open "0" commitment for output
        }
    }
}

pub struct GRR2withDelbreg;

impl GCScheme for GRR2withDelbreg {
    fn core_metrics(gate_count: &GateCount) -> Metrics {
        let total_gate_count = gate_count.and
            + gate_count.or
            + gate_count.xor
            + gate_count.nand
            + gate_count.not
            + gate_count.xnor
            + gate_count.nimp
            + gate_count.nsor;
        let xor_gate_count = gate_count.xor + gate_count.xnor;
        let non_xor_gate_count = total_gate_count - xor_gate_count;

        // assuming we use 16 byte-hash
        let hash_size = 16;
        Metrics {
            communication: 2 * hash_size * non_xor_gate_count, // 2 hashes for each non-xor gate
            computation: format!("{}H, H is hash operation", 3 * non_xor_gate_count), // 3 hash operation for each non-xor gate
            proving_cost: format!("none"), // no proving and verifying is needed,
            verifying_cost: format!("almost none"),
            assert_size: 32 * 148 * 8, // use sha256 which needs 32byte witness as commitment scheme
            disprove_size: 372956, // each gate should be put into one disprove script, which is less than 372956 bytes
        }
    }
}

pub struct GRR2withCutAndChoose;

const CUT_AND_CHOOSE_ALL: usize = 116; // parameter for cut-and-choose, how many instances are generated

impl GCScheme for GRR2withCutAndChoose {
    fn core_metrics(gate_count: &GateCount) -> Metrics {
        let total_gate_count = gate_count.and
            + gate_count.or
            + gate_count.xor
            + gate_count.nand
            + gate_count.not
            + gate_count.xnor
            + gate_count.nimp
            + gate_count.nsor;
        let xor_gate_count = gate_count.xor + gate_count.xnor;
        let non_xor_gate_count = total_gate_count - xor_gate_count;

        // assuming we use 16 byte-hash
        let hash_size = 16;
        Metrics {
            communication: 2 * hash_size * non_xor_gate_count * CUT_AND_CHOOSE_ALL, // 2 hashes for each non-xor gate
            computation: format!(
                "{}H, H is hash operation",
                3 * non_xor_gate_count * CUT_AND_CHOOSE_ALL
            ), // 3 hash operation for each non-xor gate
            proving_cost: format!("almost none"), // need to prove the connection between the input wires of the cut-and-choose instances,
            verifying_cost: format!("almost none"),
            assert_size: 32 * 148 * 8, // use sha256(32byte witness) as commitment scheme
            disprove_size: 32,         // challenger only need to open "0" commitment for output
        }
    }
}
