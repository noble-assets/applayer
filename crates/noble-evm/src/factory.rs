//! Noble EVM factory that extends EvEvmFactory with Noble-specific configuration.
//!
//! Wraps ev-reth's factory to inject custom precompiles.

use alloy_evm::{
    Database, EthEvmFactory, EvmEnv, EvmFactory,
    eth::EthEvmContext,
    precompiles::PrecompilesMap,
    revm::{
        Inspector,
        context::{
            TxEnv,
            result::{EVMError, HaltReason, InvalidTransaction},
        },
        inspector::NoOpInspector,
        primitives::hardfork::SpecId,
    },
};
use ev_revm::{EvEvm, EvEvmFactory};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct NobleEvmFactory {
    inner: EvEvmFactory<EthEvmFactory>,
}

impl NobleEvmFactory {
    /// Create a new Noble EVM factory wrapping the given EvEvmFactory.
    pub const fn new(inner: EvEvmFactory<EthEvmFactory>) -> Self {
        Self { inner }
    }

    /// Install Noble-specific precompiles into the precompiles map.
    fn install_noble_precompiles(&self, precompiles: &mut PrecompilesMap) {
        // Here we add all the precompiles that we want to include in the EVM.
        self.install_transfer_precompile(precompiles);
    }

    /// Install the TRANSFER precompile.
    fn install_transfer_precompile(&self, _precompiles: &mut PrecompilesMap) {
        // TODO: Once we have the transfer precompile we'll add it here. For now, this is a placeholder to show how precompiles would be added.
    }

    pub fn into_inner(self) -> EvEvmFactory<EthEvmFactory> {
        self.inner
    }
}

impl EvmFactory for NobleEvmFactory {
    type Evm<DB: Database, I: Inspector<Self::Context<DB>>> =
        EvEvm<EthEvmContext<DB>, I, PrecompilesMap>;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: std::error::Error + Send + Sync + 'static> =
        EVMError<DBError, InvalidTransaction>;
    type HaltReason = HaltReason;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        evm_env: EvmEnv<Self::Spec>,
    ) -> Self::Evm<DB, NoOpInspector> {
        // Create EVM with ev-reth's factory (includes their precompiles)
        let mut evm = self.inner.create_evm(db, evm_env);

        // Add our Noble-specific precompiles
        self.install_noble_precompiles(&mut evm.precompiles);

        evm
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: EvmEnv<Self::Spec>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        // Create EVM with ev-reth's factory (includes their precompiles)
        let mut evm = self.inner.create_evm_with_inspector(db, input, inspector);

        // Add our Noble-specific precompiles
        self.install_noble_precompiles(&mut evm.precompiles);

        evm
    }
}
