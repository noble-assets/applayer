use ev_node::{EvolveNode, log_startup};
use evolve_ev_reth::rpc::txpool::EvolveTxpoolApiServer;
use evolve_ev_reth::{EvolveConfig, rpc::EvolveTxpoolApiImpl};
use noble_evm::components::NobleNodeAddOns;
use reth_ethereum::cli::Cli;

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

fn main() {
    reth_cli_util::sigsegv_handler::install();

    if let Err(err) = Cli::parse_args().run(|builder, _| async move {
        log_startup();

        let handle = builder
            .with_types::<EvolveNode>()
            .with_components(noble_evm::components::noble_components_builder())
            .with_add_ons::<NobleNodeAddOns<_>>(NobleNodeAddOns::default())
            .extend_rpc_modules(move |ctx| {
                // Build custom txpool RPC with config + optional CLI/env override
                let evolve_cfg = EvolveConfig::default();
                let evolve_txpool =
                    EvolveTxpoolApiImpl::new(ctx.pool().clone(), evolve_cfg.max_txpool_bytes);

                // Merge into all enabled transports (HTTP / WS)
                ctx.modules.merge_configured(evolve_txpool.into_rpc())?;
                Ok(())
            })
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
