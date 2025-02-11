//! To build the binary:
//!
//!     `cargo build --release --bin next_header`
//!
//! To build the circuit:
//!
//!     `./target/release/circuit_function_field build`
//!
//! To prove the circuit using evm io:
//!
//!    `./target/release/circuit_function_evm prove --input-json src/bin/circuit_function_evm_input.json`
//!
//! Note that this circuit will not work with field-based io.
//!
//!
//!
use blobstreamx::next_header::CombinedStepCircuit;
use plonky2x::backend::function::Plonky2xFunction;
use tendermintx::config::{CelestiaConfig, CELESTIA_CHAIN_ID_SIZE_BYTES};

fn main() {
    const VALIDATOR_SET_SIZE_MAX: usize = 100;
    CombinedStepCircuit::<VALIDATOR_SET_SIZE_MAX, CELESTIA_CHAIN_ID_SIZE_BYTES, CelestiaConfig>::entrypoint();
}
