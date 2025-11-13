//! Helper for building Noble node components.
//!
//! This constructs a `ComponentsBuilder` configured with Noble's custom
//! executor and payload builder, ensuring Noble's precompiles and payload
//! logic are used throughout the node. This is required because the type system requires
//! consistent types for the executor and payload builder EVM configs.

use ev_node::{EvolveEngineValidatorBuilder, EvolveNode};
use evolve_ev_reth::EvolveConsensusBuilder;
use reth_ethereum::node::{
    EthereumEthApiBuilder,
    api::FullNodeTypes,
    builder::{
        components::{BasicPayloadServiceBuilder, ComponentsBuilder},
        rpc::RpcAddOns,
    },
    node::{EthereumNetworkBuilder, EthereumPoolBuilder},
};

use crate::{executor::NobleExecutorBuilder, payload::NoblePayloadBuilderBuilder};

/// Noble node addons - same as Evolve.
pub type NobleNodeAddOns<N> = RpcAddOns<N, EthereumEthApiBuilder, EvolveEngineValidatorBuilder>;

/// Build Noble components with custom executor.
pub fn noble_components_builder<N>() -> ComponentsBuilder<
    N,
    EthereumPoolBuilder,
    BasicPayloadServiceBuilder<NoblePayloadBuilderBuilder>,
    EthereumNetworkBuilder,
    NobleExecutorBuilder,
    EvolveConsensusBuilder,
>
where
    N: FullNodeTypes<Types = EvolveNode>,
{
    ComponentsBuilder::default()
        .node_types::<N>()
        .pool(EthereumPoolBuilder::default())
        .executor(NobleExecutorBuilder::default())
        .payload(BasicPayloadServiceBuilder::new(
            NoblePayloadBuilderBuilder::new(),
        ))
        .network(EthereumNetworkBuilder::default())
        .consensus(evolve_ev_reth::consensus::EvolveConsensusBuilder::default())
}
