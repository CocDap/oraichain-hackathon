use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::{ProposalStatus, State};
use cosmwasm_std::{Uint128, Addr};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom:String
}

#[cw_serde]
pub enum ExecuteMsg {
    Vote {proposal_id: Uint128, yes_vote: bool},
    Review {proposal_id: Uint128, approved: bool},
    EndVote{proposal_id: Uint128},
    EndReview{proposal_id: Uint128},
    FundingProposal {
        proposal_id: Uint128,
    },


}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    Config {},
    #[returns(GetProposalResponse)]
    GetProposal {proposal_id: Uint128},
    #[returns(TokenStakeResponse)]
    TokenStake {address: Addr},
}


#[cw_serde]
pub struct GetProposalResponse {
    pub status: ProposalStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
}

#[cw_serde]
pub struct TokenStakeResponse {
    pub token_balance: Uint128,
}


