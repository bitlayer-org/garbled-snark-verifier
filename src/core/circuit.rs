use crate::{
    bag::*,
    core::{gate::GateCount, gc_scheme_metrics::GCScheme},
};
use std::any::type_name;

pub struct Circuit(pub Wires, pub Vec<Gate>);

impl Circuit {
    pub fn empty() -> Self {
        Self(Vec::new(), Vec::new())
    }

    pub fn new(wires: Wires, gates: Vec<Gate>) -> Self {
        Self(wires, gates)
    }

    pub fn garbled_gates(&self) -> Vec<Vec<S>> {
        self.1.iter().map(|gate| gate.garbled()).collect()
    }

    pub fn extend(&mut self, circuit: Self) -> Wires {
        self.1.extend(circuit.1);
        circuit.0
    }

    pub fn add(&mut self, gate: Gate) {
        self.1.push(gate);
    }

    pub fn add_wire(&mut self, wire: Wirex) {
        self.0.push(wire);
    }

    pub fn add_wires(&mut self, wires: Wires) {
        self.0.extend(wires);
    }

    pub fn gate_count(&self) -> usize {
        self.1.len()
    }

    pub fn gate_counts(&self) -> GateCount {
        let mut and = 0;
        let mut or = 0;
        let mut xor = 0;
        let mut nand = 0;
        let mut not = 0;
        let mut xnor = 0;
        let mut nimp = 0;
        let mut nsor = 0;
        for gate in self.1.clone() {
            match gate.name.as_str() {
                "and" => and += 1,
                "or" => or += 1,
                "xor" => xor += 1,
                "nand" => nand += 1,
                "inv" | "not" => not += 1,
                "xnor" => xnor += 1,
                "nimp" => nimp += 1,
                "nsor" => nsor += 1,
                _ => panic!("this gate type is not allowed"),
            }
        }
        GateCount {
            and,
            or,
            xor,
            nand,
            not,
            xnor,
            nimp,
            nsor,
        }
    }

    /// metrics
    pub fn print_metrics<T: GCScheme>(gate_count: &GateCount) {
        println!("{}, {:?}", type_name::<T>(), T::core_metrics(gate_count))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{bristol::parser, s::S};
    use bitvm::bigint::U256;
    use bitvm::treepp::*;
    use rand::{Rng, rng};
    use serial_test::serial;
    use std::iter::zip;

    fn test_circuit(circuit_filename: &str, correct: bool) {
        println!("testing {:?}", circuit_filename);
        let (circuit, inputs, _outputs) = parser(circuit_filename);

        let mut garbled_gates = circuit.garbled_gates();
        let n = garbled_gates.len();

        if !correct {
            let u: u32 = rng().random();
            garbled_gates[(u as usize) % n] =
                vec![S::random(), S::random(), S::random(), S::random()];
        }

        for input in inputs {
            for input_wire in input {
                input_wire.borrow_mut().set(rng().random());
            }
        }

        println!(
            "testing {:?} garble",
            if correct { "correct" } else { "incorrect" }
        );

        for (i, (gate, garble)) in zip(circuit.1.clone(), garbled_gates).enumerate() {
            let a = gate.wire_a.borrow().get_label();
            let b = gate.wire_b.borrow().get_label();
            let bit_a = gate.wire_a.borrow().get_value();
            let bit_b = gate.wire_b.borrow().get_value();
            let bit_c = (gate.f())(bit_a, bit_b);
            let (garble_check, c) = gate.check_garble(garble.clone(), bit_c);
            let gate_script = gate.script(garble, garble_check);

            println!(
                "testing gate[{:?}], garble is {:?}",
                i,
                if garble_check { "correct" } else { "incorrect" }
            );

            let script = script! {
                { U256::push_hex(&hex::encode(a.0)) }
                { if bit_a {1} else {0} }
                { U256::push_hex(&hex::encode(b.0)) }
                { if bit_b {1} else {0} }
                { gate_script }
            };
            let result = execute_script(script);
            assert!(result.success);

            if garble_check {
                gate.wire_c.borrow_mut().set2(bit_c, c);
            } else {
                assert!(!correct);
                break;
            }
        }
    }

    fn test_circuit_find_incorrect(circuit_filename: &str, correct: bool) {
        println!("testing {:?}", circuit_filename);
        let (circuit, inputs, _outputs) = parser(circuit_filename);

        let mut garbled_gates = circuit.garbled_gates();
        let n = garbled_gates.len();

        if !correct {
            let u: u32 = rng().random();
            garbled_gates[(u as usize) % n] =
                vec![S::random(), S::random(), S::random(), S::random()];
        }

        for input in inputs {
            for input_wire in input {
                input_wire.borrow_mut().set(rng().random());
            }
        }

        println!(
            "testing {:?} garble",
            if correct { "correct" } else { "incorrect" }
        );

        for (i, (gate, garble)) in zip(circuit.1.clone(), garbled_gates).enumerate() {
            let a = gate.wire_a.borrow().get_label();
            let b = gate.wire_b.borrow().get_label();
            let bit_a = gate.wire_a.borrow().get_value();
            let bit_b = gate.wire_b.borrow().get_value();
            let bit_c = (gate.f())(bit_a, bit_b);
            let (garble_check, c) = gate.check_garble(garble.clone(), bit_c);

            println!(
                "testing gate[{:?}], garble is {:?}",
                i,
                if garble_check { "correct" } else { "incorrect" }
            );

            if garble_check {
                gate.wire_c.borrow_mut().set2(bit_c, c);
                continue;
            }
            assert!(!correct);

            let gate_script = gate.script(garble, garble_check);

            let script = script! {
                { U256::push_hex(&hex::encode(a.0)) }
                { if bit_a {1} else {0} }
                { U256::push_hex(&hex::encode(b.0)) }
                { if bit_b {1} else {0} }
                { gate_script }
            };
            let result = execute_script(script);
            assert!(result.success);

            break;
        }
    }

    #[test]
    #[serial]
    fn test_circuit_adder() {
        test_circuit("src/core/bristol-examples/adder64.txt", true);
        test_circuit("src/core/bristol-examples/adder64.txt", false);
    }

    #[test]
    #[serial]
    fn test_circuit_adder_find_incorrect() {
        test_circuit_find_incorrect("src/core/bristol-examples/adder64.txt", true);
        test_circuit_find_incorrect("src/core/bristol-examples/adder64.txt", false);
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_circuit_subtracter() {
        test_circuit("src/core/bristol-examples/subtracter64.txt", true);
        test_circuit("src/core/bristol-examples/subtracter64.txt", false);
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_circuit_subtracter_find_incorrect() {
        test_circuit_find_incorrect("src/core/bristol-examples/subtracter64.txt", true);
        test_circuit_find_incorrect("src/core/bristol-examples/subtracter64.txt", false);
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_circuit_multiplier() {
        test_circuit("src/core/bristol-examples/multiplier64.txt", true);
        test_circuit("src/core/bristol-examples/multiplier64.txt", false);
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_circuit_multiplier_find_incorrect() {
        test_circuit_find_incorrect("src/core/bristol-examples/multiplier64.txt", true);
        test_circuit_find_incorrect("src/core/bristol-examples/multiplier64.txt", false);
    }
}
