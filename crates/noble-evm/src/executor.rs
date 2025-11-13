//! Noble executor builder that uses NobleEvmFactory.
//!
//! This wraps ev-reth's executor to inject our NobleEvmFactory, ensuring Noble's
//! custom precompiles are available during block execution. The wrapper is necessary
//! because Rust's type system requires the executor's EVM config type to match the
//! payload builder's expected type throughout the entire component stack.

use crate::factory::NobleEvmFactory;
use alloy_evm::eth::spec::EthExecutorSpec;
use ev_node::EvolveNode;
use reth_ethereum::{
    chainspec::{ChainSpec, EthereumHardforks},
    evm::EthEvmConfig,
    node::{
        api::FullNodeTypes,
        builder::{BuilderContext, components::ExecutorBuilder},
    },
};
use reth_ethereum_forks::Hardforks;
use tracing::info;

/// Type alias for the Noble-aware EVM config.
pub type NobleEvmConfig = EthEvmConfig<ChainSpec, NobleEvmFactory>;

/// Noble executor builder that constructs EVMs with Noble precompiles.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct NobleExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for NobleExecutorBuilder
where
    Node: FullNodeTypes<Types = EvolveNode>,
    ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
{
    type EVM = NobleEvmConfig;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        info!(target: "noble-evm", "Building Noble EVM...");

        // Build the base Evolve EVM config
        let base_config = ev_node::executor::build_evm_config(ctx)?;

        // Extract the EvEvmFactory from the base config
        let ev_factory = base_config.executor_factory.evm_factory().clone();

        // Wrap with Noble factory (adds our additional configuration)
        let noble_factory = NobleEvmFactory::new(ev_factory);

        // Reconstruct the EVM config with Noble factory
        Ok(EthEvmConfig {
            executor_factory: alloy_evm::eth::EthBlockExecutorFactory::new(
                *base_config.executor_factory.receipt_builder(),
                base_config.executor_factory.spec().clone(),
                noble_factory,
            ),
            block_assembler: base_config.block_assembler,
        })
    }
}
