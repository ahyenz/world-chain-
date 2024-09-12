use reth_basic_payload_builder::{BasicPayloadJobGenerator, BasicPayloadJobGeneratorConfig};
use reth_chainspec::ChainSpec;
use reth_evm::ConfigureEvm;
use reth_node_builder::{
    components::{ComponentsBuilder, PayloadServiceBuilder},
    BuilderContext, FullNodeTypes, Node, NodeTypes, NodeTypesWithEngine, PayloadBuilderConfig,
};
use reth_node_optimism::{
    args::RollupArgs,
    node::{OptimismAddOns, OptimismConsensusBuilder},
    OptimismEngineTypes, OptimismEvmConfig,
};
use reth_payload_builder::{PayloadBuilderHandle, PayloadBuilderService};
use reth_provider::CanonStateSubscriptions;
use reth_transaction_pool::TransactionPool;

use crate::{
    executer::builder::WcExecutorBuilder,
    network::builder::WcNetworkBuilder,
    payload::builder::PBHBuilder,
    pool::builder::WcPoolBuilder, // payload::PBHBuilder,
};

use super::args::{ExtArgs, WcBuilderArgs};

#[derive(Debug, Clone)]
pub struct WorldChainBuilder {
    /// Additional Optimism args
    pub args: ExtArgs,
}

impl WorldChainBuilder {
    pub const fn new(args: ExtArgs) -> Self {
        Self { args }
    }

    /// Returns the components for the given [`RollupArgs`].
    pub fn components<Node>(
        args: ExtArgs,
    ) -> ComponentsBuilder<
        Node,
        WcPoolBuilder,
        WcPayloadBuilder,
        WcNetworkBuilder,
        WcExecutorBuilder,
        OptimismConsensusBuilder,
    >
    where
        Node: FullNodeTypes<
            Types: NodeTypesWithEngine<Engine = OptimismEngineTypes, ChainSpec = ChainSpec>,
        >,
    {
        let WcBuilderArgs { clear_nullifiers } = args.builder_args;
        let RollupArgs {
            disable_txpool_gossip,
            discovery_v4,
            ..
        } = args.rollup_args;
        ComponentsBuilder::default()
            .node_types::<Node>()
            .pool(WcPoolBuilder { clear_nullifiers })
            .payload(WcPayloadBuilder::new(OptimismEvmConfig::default()))
            .network(WcNetworkBuilder {
                disable_txpool_gossip,
                disable_discovery_v4: !discovery_v4,
            })
            .executor(WcExecutorBuilder::default())
            .consensus(OptimismConsensusBuilder::default())
    }
}

impl<N> Node<N> for WorldChainBuilder
where
    N: FullNodeTypes<
        Types: NodeTypesWithEngine<Engine = OptimismEngineTypes, ChainSpec = ChainSpec>,
    >,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        WcPoolBuilder,
        WcPayloadBuilder,
        WcNetworkBuilder,
        WcExecutorBuilder,
        OptimismConsensusBuilder,
    >;

    type AddOns = OptimismAddOns;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        let Self { args } = self;
        Self::components(args.clone())
    }
}

impl NodeTypes for WorldChainBuilder {
    type Primitives = ();
    type ChainSpec = ChainSpec;
}

impl NodeTypesWithEngine for WorldChainBuilder {
    type Engine = OptimismEngineTypes;
}

#[derive(Debug, Default, Clone)]
pub struct WcPayloadBuilder<EVM = OptimismEvmConfig> {
    /// The EVM configuration to use for the payload builder.
    pub evm_config: EVM,
}

impl<EVM> WcPayloadBuilder<EVM> {
    pub const fn new(evm_config: EVM) -> Self {
        Self { evm_config }
    }
}

impl<Node, EVM, Pool> PayloadServiceBuilder<Node, Pool> for WcPayloadBuilder<EVM>
where
    Node: FullNodeTypes<
        Types: NodeTypesWithEngine<Engine = OptimismEngineTypes, ChainSpec = ChainSpec>,
    >,
    Pool: TransactionPool + Unpin + 'static,
    EVM: ConfigureEvm,
{
    async fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<PayloadBuilderHandle<OptimismEngineTypes>> {
        let payload_builder = PBHBuilder::new(self.evm_config);

        let conf = ctx.payload_builder_config();

        let payload_job_config = BasicPayloadJobGeneratorConfig::default()
            .interval(conf.interval())
            .deadline(conf.deadline())
            .max_payload_tasks(conf.max_payload_tasks())
            // no extradata for OP
            .extradata(Default::default());

        let payload_generator = BasicPayloadJobGenerator::with_builder(
            ctx.provider().clone(),
            pool,
            ctx.task_executor().clone(),
            payload_job_config,
            ctx.chain_spec(),
            payload_builder,
        );
        let (payload_service, payload_builder) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream());

        ctx.task_executor()
            .spawn_critical("payload builder service", Box::pin(payload_service));

        Ok(payload_builder)
    }
}
