//! Noble engine payload builder - wraps EvolvePayloadBuilder.
//!
//! This allows us to use the Evolve payload building logic while
//! ensuring that our NobleEvmFactory (with Noble precompiles) is used
//! during block execution.
//! Unfortunately a lot of the logic had to be re-implemented here
//! because the type system requires consistent types for the
//! executor and payload builder EVM configs.

use std::sync::Arc;

use alloy_primitives::{Address, U256};
use ev_node::{
    EvolveEnginePayloadBuilderAttributes, EvolveEngineTypes, EvolvePayloadBuilder,
    EvolvePayloadBuilderConfig,
};
use evolve_ev_reth::EvolvePayloadAttributes;
use eyre::WrapErr;
use reth_basic_payload_builder::{
    BuildArguments, BuildOutcome, HeaderForPayload, MissingPayloadBehaviour, PayloadBuilder,
    PayloadConfig,
};
use reth_ethereum::{
    TransactionSigned,
    chainspec::{ChainSpec, ChainSpecProvider},
    node::{
        api::{FullNodeTypes, NodeTypes, payload::PayloadBuilderAttributes},
        builder::{BuilderContext, components::PayloadBuilderBuilder},
    },
    pool::{PoolTransaction, TransactionPool},
    primitives::Header,
};
use reth_payload_builder::{EthBuiltPayload, PayloadBuilderError};
use reth_provider::HeaderProvider;
use reth_revm::cached::CachedReads;
use tokio::runtime::Handle;
use tracing::info;

use crate::executor::NobleEvmConfig;
use evolve_ev_reth::config::set_current_block_gas_limit;

/// Noble payload service builder.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NoblePayloadBuilderBuilder {
    config: EvolvePayloadBuilderConfig,
}

impl NoblePayloadBuilderBuilder {
    pub fn new() -> Self {
        let config = EvolvePayloadBuilderConfig::new();
        info!("Created Noble payload builder with config: {:?}", config);
        Self { config }
    }
}

impl Default for NoblePayloadBuilderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The Noble engine payload builder - wraps EvolvePayloadBuilder.
#[derive(Debug, Clone)]
pub struct NobleEnginePayloadBuilder<Client>
where
    Client: Clone,
{
    pub(crate) evolve_builder: Arc<EvolvePayloadBuilder<Client>>,
    pub(crate) config: EvolvePayloadBuilderConfig,
}

impl<Node, Pool> PayloadBuilderBuilder<Node, Pool, NobleEvmConfig> for NoblePayloadBuilderBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            Payload = EvolveEngineTypes,
            ChainSpec = ChainSpec,
            Primitives = reth_ethereum::EthPrimitives,
        >,
    >,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
{
    type PayloadBuilder = NobleEnginePayloadBuilder<Node::Provider>;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        _pool: Pool,
        evm_config: NobleEvmConfig,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let chain_spec = ctx.chain_spec();
        let mut config = EvolvePayloadBuilderConfig::from_chain_spec(&chain_spec)
            .wrap_err("failed to load evolve config from chain spec")?;

        if self.config.base_fee_sink.is_some() {
            config.base_fee_sink = self.config.base_fee_sink;
        }

        config.validate()?;

        // SAFETY: NobleEvmConfig and EvolveEvmConfig have identical memory layouts.
        // Both are EthEvmConfig<ChainSpec, Factory> where:
        // - NobleEvmConfig uses NobleEvmFactory (which wraps EvEvmFactory<EthEvmFactory>)
        // - EvolveEvmConfig uses EvEvmFactory<EthEvmFactory>
        // Since NobleEvmFactory is #[repr(transparent)], the layouts match.
        // Therefore, transmuting between them is safe.
        // This saves us a lot of boilerplate in re-implementing a NoblePayloadBuilder
        // that uses NobleEvmConfig.
        let evolve_evm_config: ev_node::executor::EvolveEvmConfig =
            unsafe { std::mem::transmute(evm_config) };

        let evolve_builder = Arc::new(EvolvePayloadBuilder::new(
            Arc::new(ctx.provider().clone()),
            evolve_evm_config,
            config.clone(),
        ));

        Ok(NobleEnginePayloadBuilder {
            evolve_builder,
            config,
        })
    }
}

impl<Client> PayloadBuilder for NobleEnginePayloadBuilder<Client>
where
    Client: reth_ethereum::provider::StateProviderFactory
        + ChainSpecProvider<ChainSpec = ChainSpec>
        + HeaderProvider<Header = Header>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Attributes = EvolveEnginePayloadBuilderAttributes;
    type BuiltPayload = EthBuiltPayload;

    fn try_build(
        &self,
        args: BuildArguments<Self::Attributes, Self::BuiltPayload>,
    ) -> Result<BuildOutcome<Self::BuiltPayload>, PayloadBuilderError> {
        let BuildArguments {
            cached_reads: _,
            config,
            cancel: _,
            best_payload: _,
        } = args;
        let PayloadConfig {
            parent_header,
            attributes,
        } = config;

        info!(
            "Noble engine payload builder: building payload with {} transactions",
            attributes.transactions.len()
        );

        // Convert Engine API attributes to Evolve payload attributes.
        // If no gas_limit provided, default to the parent header's gas limit (genesis for first block).
        let effective_gas_limit = attributes.gas_limit.unwrap_or(parent_header.gas_limit);
        // Publish effective gas limit for RPC alignment.
        set_current_block_gas_limit(effective_gas_limit);

        let mut fee_recipient = attributes.suggested_fee_recipient();
        if fee_recipient == Address::ZERO
            && let Some(sink) = self.config.base_fee_sink
        {
            info!(
                target: "noble-evm",
                fee_sink = ?sink,
                "Suggested fee recipient missing; defaulting to base-fee sink"
            );
            fee_recipient = sink;
        }

        let evolve_attrs = EvolvePayloadAttributes::new(
            attributes.transactions.clone(),
            Some(effective_gas_limit),
            attributes.timestamp(),
            attributes.prev_randao(),
            fee_recipient,
            attributes.parent(),
            parent_header.number + 1,
        );

        // Build the payload using the evolve payload builder - use spawn_blocking for async work.
        let evolve_builder = self.evolve_builder.clone();
        let sealed_block = tokio::task::block_in_place(|| {
            Handle::current().block_on(evolve_builder.build_payload(evolve_attrs))
        })
        .map_err(PayloadBuilderError::other)?;

        info!(
            "Noble engine payload builder: built block with {} transactions, gas used: {}",
            sealed_block.transaction_count(),
            sealed_block.gas_used
        );

        // Convert to EthBuiltPayload.
        let gas_used = sealed_block.gas_used;
        let built_payload = EthBuiltPayload::new(
            attributes.payload_id(), // Use the proper payload ID from attributes.
            Arc::new(sealed_block),
            U256::from(gas_used), // Block gas used.
            None,                 // No blob sidecar for noble.
        );

        Ok(BuildOutcome::Better {
            payload: built_payload,
            cached_reads: CachedReads::default(),
        })
    }

    fn build_empty_payload(
        &self,
        config: PayloadConfig<Self::Attributes, HeaderForPayload<Self::BuiltPayload>>,
    ) -> Result<Self::BuiltPayload, PayloadBuilderError> {
        let PayloadConfig {
            parent_header,
            attributes,
        } = config;

        info!("Noble engine payload builder: building empty payload");

        // Create empty noble attributes (no transactions).
        // If no gas_limit provided, default to the parent header's gas limit (genesis for first block).
        let effective_gas_limit = attributes.gas_limit.unwrap_or(parent_header.gas_limit);
        // Publish effective gas limit for RPC alignment.
        set_current_block_gas_limit(effective_gas_limit);

        let mut fee_recipient = attributes.suggested_fee_recipient();
        if fee_recipient == Address::ZERO
            && let Some(sink) = self.config.base_fee_sink
        {
            info!(
                target: "noble-evm",
                fee_sink = ?sink,
                "Suggested fee recipient missing; defaulting to base-fee sink"
            );
            fee_recipient = sink;
        }

        let evolve_attrs = EvolvePayloadAttributes::new(
            vec![],
            Some(effective_gas_limit),
            attributes.timestamp(),
            attributes.prev_randao(),
            fee_recipient,
            attributes.parent(),
            parent_header.number + 1,
        );

        // Build empty payload - use spawn_blocking for async work.
        let evolve_builder = self.evolve_builder.clone();
        let sealed_block = tokio::task::block_in_place(|| {
            Handle::current().block_on(evolve_builder.build_payload(evolve_attrs))
        })
        .map_err(PayloadBuilderError::other)?;

        let gas_used = sealed_block.gas_used;
        Ok(EthBuiltPayload::new(
            attributes.payload_id(),
            Arc::new(sealed_block),
            U256::from(gas_used),
            None,
        ))
    }

    /// Determines how to handle a request for a payload that is currently being built.
    ///
    /// This will always await the in-progress job, preventing a race with a new build.
    /// This is the recommended behavior to prevent redundant payload builds.
    fn on_missing_payload(
        &self,
        _args: BuildArguments<Self::Attributes, Self::BuiltPayload>,
    ) -> MissingPayloadBehaviour<Self::BuiltPayload> {
        MissingPayloadBehaviour::AwaitInProgress
    }
}
