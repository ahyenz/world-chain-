use alloy_primitives::Address;
use clap::value_parser;
use reth_optimism_node::args::RollupArgs;

#[derive(Debug, Clone, Default, PartialEq, Eq, clap::Args)]
#[command(next_help_heading = "PBH Builder")]
pub struct WorldChainArgs {
    /// op rollup args
    #[command(flatten)]
    pub rollup_args: RollupArgs,

    /// Sets the number of allowed PBH transactions per month
    #[arg(long = "builder.num_pbh_txs", default_value = "30")]
    pub num_pbh_txs: u8,

    /// Sets the max blockspace reserved for verified transactions. If there are not enough
    /// verified transactions to fill the capacity, the remaining blockspace will be filled with
    /// unverified transactions.
    /// This arg is a percentage of the total blockspace with the default set to 70 (ie 70%).
    #[arg(long = "builder.verified_blockspace_capacity", default_value = "70", value_parser = value_parser!(u8).range(0..=100))]
    pub verified_blockspace_capacity: u8,

    /// Sets the ERC-4337 EntryPoint Proxy contract address
    /// This contract is used to validate 4337 PBH bundles
    #[arg(long = "builder.pbh_entrypoint")]
    pub pbh_entrypoint: Address,

    /// Sets the WorldID contract address.
    /// This contract is used to provide the latest merkle root on chain.
    #[arg(
        long = "builder.world_id",
        default_value = "0x047eE5313F98E26Cc8177fA38877cB36292D2364"
    )]
    pub world_id: Address,

    /// Sets the ERC0-7766 Signature Aggregator contract address
    /// This contract signifies that a given bundle should receive priority inclusion if it passes validation
    #[arg(long = "builder.signature_aggregator")]
    pub signature_aggregator: Address,
}
