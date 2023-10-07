use celestia::skip::{SkipOffchainInputs, TendermintSkipCircuit};
use plonky2x::backend::circuit::Circuit;
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::prelude::{Bytes32Variable, CircuitBuilder, PlonkParameters};

use crate::circuit::{CelestiaDataCommitmentCircuit, DataCommitmentOffchainInputs};

pub struct CombinedSkipCircuit<const MAX_LEAVES: usize, const MAX_VALIDATOR_SET_SIZE: usize> {
    _config: usize,
}

impl<const MAX_LEAVES: usize, const MAX_VALIDATOR_SET_SIZE: usize> Circuit
    for CombinedSkipCircuit<MAX_LEAVES, MAX_VALIDATOR_SET_SIZE>
{
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let trusted_header_hash = builder.evm_read::<Bytes32Variable>();
        let trusted_block = builder.evm_read::<U64Variable>();
        let target_block = builder.evm_read::<U64Variable>();

        let target_header_hash = builder.skip_from_inputs::<MAX_VALIDATOR_SET_SIZE>(
            trusted_block,
            trusted_header_hash,
            target_block,
        );

        let data_commitment = builder.data_commitment_from_inputs::<MAX_LEAVES>(
            trusted_block,
            trusted_header_hash,
            target_block,
            target_header_hash,
        );

        builder.evm_write(target_header_hash);
        builder.evm_write(data_commitment);
    }

    fn register_generators<L: PlonkParameters<D>, const D: usize>(
        generator_registry: &mut plonky2x::prelude::HintRegistry<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<L::Field>,
    {
        generator_registry.register_async_hint::<DataCommitmentOffchainInputs<1>>();
        generator_registry.register_async_hint::<SkipOffchainInputs<MAX_VALIDATOR_SET_SIZE>>();
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use ethers::types::H256;
    use plonky2x::prelude::{DefaultBuilder, GateRegistry, HintRegistry};
    use subtle_encoding::hex;

    use super::*;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_data_commitment_serialization() {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        const MAX_LEAVES: usize = 2;
        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        DataCommitmentCircuit::<MAX_LEAVES>::define(&mut builder);
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut hint_registry = HintRegistry::new();
        let mut gate_registry = GateRegistry::new();
        DataCommitmentCircuit::<MAX_LEAVES>::register_generators(&mut hint_registry);
        DataCommitmentCircuit::<MAX_LEAVES>::register_gates(&mut gate_registry);

        circuit.test_serializers(&gate_registry, &hint_registry);
    }

    fn test_data_commitment_template<const MAX_LEAVES: usize>(
        start_block: usize,
        start_header_hash: [u8; 32],
        end_block: usize,
        end_header_hash: [u8; 32],
    ) {
        env::set_var("RUST_LOG", "debug");
        env_logger::try_init().unwrap_or_default();

        // env::set_var("RPC_MOCHA_4", "fixture"); // Use fixture during testing

        let mut builder = DefaultBuilder::new();

        log::debug!("Defining circuit");
        DataCommitmentCircuit::<MAX_LEAVES>::define(&mut builder);

        log::debug!("Building circuit");
        let circuit = builder.build();
        log::debug!("Done building circuit");

        let mut input = circuit.input();

        input.evm_write::<U64Variable>(start_block as u64);
        input.evm_write::<Bytes32Variable>(H256::from_slice(start_header_hash.as_slice()));
        input.evm_write::<U64Variable>(end_block as u64);
        input.evm_write::<Bytes32Variable>(H256::from_slice(end_header_hash.as_slice()));

        log::debug!("Generating proof");

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (proof, mut output) = rt.block_on(async { circuit.prove_async(&input).await });

        log::debug!("Done generating proof");

        circuit.verify(&proof, &input, &output);
        let data_commitment = output.evm_read::<Bytes32Variable>();
        println!("data_commitment {:?}", data_commitment);
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_data_commitment_small() {
        // Test variable length NUM_BLOCKS.
        const MAX_LEAVES: usize = 8;
        const NUM_BLOCKS: usize = 4;

        let start_block = 10000u64;
        let start_header_hash =
            hex::decode_upper("A0123D5E4B8B8888A61F931EE2252D83568B97C223E0ECA9795B29B8BD8CBA2D")
                .unwrap();
        let end_block = start_block + NUM_BLOCKS as u64;
        let end_header_hash =
            hex::decode_upper("FCDA37FA6306C77737DD911E6101B612E2DBD837F29ED4F4E1C30919FBAC9D05")
                .unwrap();

        test_data_commitment_template::<MAX_LEAVES>(
            start_block as usize,
            start_header_hash.as_slice().try_into().unwrap(),
            end_block as usize,
            end_header_hash.as_slice().try_into().unwrap(),
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_data_commitment_large() {
        // Test variable length NUM_BLOCKS.
        const MAX_LEAVES: usize = 1024;
        const NUM_BLOCKS: usize = 4;

        let start_block = 10000u64;
        let start_header_hash =
            hex::decode_upper("A0123D5E4B8B8888A61F931EE2252D83568B97C223E0ECA9795B29B8BD8CBA2D")
                .unwrap();
        let end_block = start_block + NUM_BLOCKS as u64;
        let end_header_hash =
            hex::decode_upper("FCDA37FA6306C77737DD911E6101B612E2DBD837F29ED4F4E1C30919FBAC9D05")
                .unwrap();

        test_data_commitment_template::<MAX_LEAVES>(
            start_block as usize,
            start_header_hash.as_slice().try_into().unwrap(),
            end_block as usize,
            end_header_hash.as_slice().try_into().unwrap(),
        );
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_data_commitment_smart_contract() {
        // Test variable length NUM_BLOCKS.
        const MAX_LEAVES: usize = 256;
        const NUM_BLOCKS: usize = 4;

        let start_block = 10000u64;
        let start_header_hash =
            hex::decode_upper("A0123D5E4B8B8888A61F931EE2252D83568B97C223E0ECA9795B29B8BD8CBA2D")
                .unwrap();
        let end_block = start_block + NUM_BLOCKS as u64;
        let end_header_hash =
            hex::decode_upper("FCDA37FA6306C77737DD911E6101B612E2DBD837F29ED4F4E1C30919FBAC9D05")
                .unwrap();

        test_data_commitment_template::<MAX_LEAVES>(
            start_block as usize,
            start_header_hash.as_slice().try_into().unwrap(),
            end_block as usize,
            end_header_hash.as_slice().try_into().unwrap(),
        );
    }
}
