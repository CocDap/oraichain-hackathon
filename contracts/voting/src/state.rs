use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub denom: String,
    pub owner: Addr,
    //pub staked_tokens:Uint128
}


#[cw_serde]
pub enum ProposalStatus {
    InProgress,
    Passed,
    Rejected,
}


#[cw_serde]
pub struct Voter {
    //pub is_voted: bool,
    //Vec (proposal_id, number of votes)
    pub vote_count: Vec<(u64,u64)>,
}


#[cw_serde]
#[derive(Default)]
pub struct BalanceVote {
    pub token_balance: Uint128,             // total staked balance
    pub locked_tokens: Vec<(u64, Uint128)>, //maps poll_id to weight voted
    pub participated_polls: Vec<u64>,       // poll_id
}


#[cw_serde]
pub struct Proposal {
    pub status: ProposalStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub voters: Vec<Addr>,
}
pub const STATE: Item<State> = Item::new("state");

pub const PROPOSALS: Map<&[u8], Proposal> = Map::new("proposals");

pub const VOTERS: Map<&[u8],Voter> = Map::new("voter");

pub const BALANCES: Map<&[u8], BalanceVote> = Map::new("balance");
