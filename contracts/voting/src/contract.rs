#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::validate_sent_sufficient_coin;
use crate::msg::{ExecuteMsg, GetProposalResponse, InstantiateMsg, QueryMsg, TokenStakeResponse};
use crate::state::{ProposalStatus, State, Voter, BALANCES, PROPOSALS, STATE, VOTERS};
use cosmwasm_std::Uint128;
// version info for migration info

pub const VOTING_TOKEN: &str = "voting_token";
const CONTRACT_NAME: &str = "crates.io:voting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MIN_STAKE_AMOUNT: u128 = 1000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        denom: msg.denom,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        } => execute::vote(deps, info, proposal_id, yes_vote),

        ExecuteMsg::EndVote { proposal_id } => execute::end_vote(deps, info, proposal_id),
        //ExecuteMsg::Withdraw {} => execute::withdraw(deps, info),
    }
}

pub mod execute {
    use crate::state::Proposal;

    use super::*;

    pub fn vote(
        deps: DepsMut,
        info: MessageInfo,
        proposal_id: Uint128,
        yes_vote: bool,
    ) -> Result<Response, ContractError> {
        let key_proposal_id = &proposal_id.to_be_bytes();

        let key_address = info.sender.as_str().as_bytes();

        let state = STATE.load(deps.storage)?;

        if let Some(mut proposal) = PROPOSALS.may_load(deps.storage, key_proposal_id)? {
            if yes_vote {
                proposal.yes_votes += Uint128::from(1u128);
            } else {
                proposal.no_votes += Uint128::from(1u128);
            }

            let mut balance_member = BALANCES
                .may_load(deps.storage, key_address)?
                .unwrap_or_default();

            let voter = VOTERS.may_load(deps.storage, key_address)?;

            match voter {
                Some(mut voter) => {
                    let stake_amount =
                        match voter.vote_count.iter().position(|&x| x.0 == proposal_id) {
                            Some(index) => {
                                println!("Go to here");
                                let next_vote = voter.vote_count.get(index).unwrap().1 + 1;

                                let insert_vote = (proposal_id, next_vote);

                                voter.vote_count.insert(index, insert_vote);

                                next_vote.checked_pow(2)
                            }
                            None => return Err(ContractError::ProposalNotFound {}),
                        };

                    println!(
                        "Staking amount:{}, funds:{:?}",
                        stake_amount.unwrap(),
                        info.funds
                    );

                    let amount = MIN_STAKE_AMOUNT
                        .checked_mul(stake_amount.unwrap() as u128)
                        .unwrap_or_default();
                    validate_sent_sufficient_coin(&info.funds, Some(coin(amount, &state.denom)))?;

                    VOTERS.save(deps.storage, key_address, &voter)?;
                }
                None => {
                    let count = 1;
                    let voter = Voter {
                        vote_count: vec![(proposal_id, count)],
                    };

                    validate_sent_sufficient_coin(
                        &info.funds,
                        Some(coin(MIN_STAKE_AMOUNT, &state.denom)),
                    )?;

                    VOTERS.save(deps.storage, key_address, &voter)?;
                }
            }

            let funds = info
                .funds
                .iter()
                .find(|coin| coin.denom.eq(&state.denom))
                .unwrap();

            balance_member.token_balance += funds.amount;

            BALANCES.save(deps.storage, key_address, &balance_member)?;
            PROPOSALS.save(deps.storage, key_proposal_id, &proposal)?;
        } else {
            // First time
            //voter 1 -> proposal 1
            // voter 2 -> proposal 1

            // voter 1 -> proposal 1
            // voter 2 -> proposal 2

            println!("Here");
            let mut yes = Uint128::from(0u128);
            let mut no = Uint128::from(0u128);
            if yes_vote {
                yes = Uint128::from(1u128);
            } else {
                no = Uint128::from(1u128);
            }
            println!("no:{}", no);
            let new_proposal = Proposal {
                status: ProposalStatus::InProgress,
                yes_votes: yes,
                no_votes: no,
                voters: Vec::new(),
            };

            let mut balance_member = BALANCES
                .may_load(deps.storage, key_address)?
                .unwrap_or_default();

            validate_sent_sufficient_coin(&info.funds, Some(coin(MIN_STAKE_AMOUNT, &state.denom)))?;

            let funds = info
                .funds
                .iter()
                .find(|coin| coin.denom.eq(&state.denom))
                .unwrap();
            println!("Funds:{}", funds);
            balance_member.token_balance += funds.amount;

            let count = 1;
            let voter = Voter {
                vote_count: vec![(proposal_id, count)],
            };
            BALANCES.save(deps.storage, key_address, &balance_member)?;

            PROPOSALS.save(deps.storage, key_proposal_id, &new_proposal)?;
            VOTERS.save(deps.storage, key_address, &voter)?;
        }
        STATE.save(deps.storage, &state)?;

        Ok(Response::new().add_attribute("action", "vote"))
    }

    pub fn end_vote(
        deps: DepsMut,
        info: MessageInfo,
        proposal_id: Uint128,
    ) -> Result<Response, ContractError> {
        let key_proposal_id = &proposal_id.to_be_bytes();
        let state = STATE.load(deps.storage)?;

        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        match PROPOSALS.may_load(deps.storage, key_proposal_id)? {
            Some(mut proposal) => {
                if proposal.status != ProposalStatus::InProgress {
                    return Err(ContractError::ProposalNotInProgress {});
                }

                if proposal.yes_votes > proposal.no_votes {
                    proposal.status = ProposalStatus::Passed;
                } else {
                    proposal.status = ProposalStatus::Rejected;
                }
                PROPOSALS.save(deps.storage, key_proposal_id, &proposal)?;
            }
            None => return Err(ContractError::ProposalNotFound {}),
        };

        Ok(Response::new().add_attribute("action", "end_vote"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&STATE.load(deps.storage)?),
        QueryMsg::TokenStake { address } => {
            query::token_balance(deps, deps.api.addr_validate(address.as_str())?)
        }
        QueryMsg::GetProposal { proposal_id } => {
            to_binary(&query::get_proposal(deps, proposal_id)?)
        }
    }
}

pub mod query {
    use super::*;

    pub fn get_proposal(deps: Deps, proposal_id: Uint128) -> StdResult<GetProposalResponse> {
        let key = &proposal_id.to_be_bytes();
        let proposal = PROPOSALS.load(deps.storage, key)?;
        Ok(GetProposalResponse {
            status: proposal.status,
            yes_votes: proposal.yes_votes,
            no_votes: proposal.no_votes,
        })
    }

    pub fn token_balance(deps: Deps, address: Addr) -> StdResult<Binary> {
        let token_manager = BALANCES
            .may_load(deps.storage, address.as_str().as_bytes())?
            .unwrap_or_default();

        let resp = TokenStakeResponse {
            token_balance: token_manager.token_balance,
        };

        to_binary(&resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    const TEST_VOTER: &str = "voter1";
    const TEST_VOTER_2: &str = "voter2";
    const TEST_VOTER_3: &str = "voter3";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: String::from(VOTING_TOKEN),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn vote() {
        const BASE: u128 = 1000;
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: String::from(VOTING_TOKEN),
        };
        let info = mock_info("creator", &coins(2*BASE, &msg.denom));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        let info = mock_info(TEST_VOTER, &coins(1*BASE, &msg.denom));
        let yes_vote = true;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1u128), value.yes_votes);
        assert_eq!(Uint128::from(0u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(1*BASE), token_stake.token_balance);

        let info = mock_info(TEST_VOTER_2, &coins(1*BASE, &msg.denom));

        let yes_vote = true;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };

        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2u128), value.yes_votes);
        assert_eq!(Uint128::from(0u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER_2.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(1*BASE), token_stake.token_balance);
        println!("Success 2");
        //Third time

        let yes_vote = false;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };

        let info = mock_info(TEST_VOTER_2, &coins(4*BASE, &msg.denom));
        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2u128), value.yes_votes);
        assert_eq!(Uint128::from(1u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER_2.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(5*BASE), token_stake.token_balance);

        //Four times
        let yes_vote = false;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };

        let info = mock_info(TEST_VOTER_2, &coins(9*BASE, &msg.denom));
        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2u128), value.yes_votes);
        assert_eq!(Uint128::from(2u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER_2.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(14*BASE), token_stake.token_balance);

        // Vote another proposal id
        let yes_vote = false;
        let proposal_id = Uint128::from(2u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };

        let info = mock_info(TEST_VOTER_3, &coins(1*BASE, &msg.denom));
        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(2u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(0u128), value.yes_votes);
        assert_eq!(Uint128::from(1u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER_3.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(1*BASE), token_stake.token_balance);

        //Voter 3 vote for proposal 2
        let yes_vote = true;
        let proposal_id = Uint128::from(2u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };

        let info = mock_info(TEST_VOTER_3, &coins(4*BASE, &msg.denom));
        let _res = execute(deps.as_mut(), mock_env(), info, msg_execute).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(2u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1u128), value.yes_votes);
        assert_eq!(Uint128::from(1u128), value.no_votes);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenStake {
                address: Addr::unchecked(TEST_VOTER_3.to_string()),
            },
        )
        .unwrap();
        let token_stake: TokenStakeResponse = from_binary(&res).unwrap();

        assert_eq!(Uint128::from(5*BASE), token_stake.token_balance);
    }

    
    #[test]
    fn end_vote_passed() {
        let mut deps = mock_dependencies();

        const BASE: u128 = 1000;
        let msg = InstantiateMsg {
            denom: String::from(VOTING_TOKEN),
        };
        let owner_info = mock_info("creator", &coins(1*BASE, &msg.denom));
        let _res = instantiate(deps.as_mut(), mock_env(), owner_info.clone(), msg.clone()).unwrap();

        let voter_info = mock_info("anyone", &coins(1*BASE, &msg.denom));

        let yes_vote = true;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };
        let _res = execute(deps.as_mut(), mock_env(), voter_info, msg_execute).unwrap();

        let msg_end_vote = ExecuteMsg::EndVote { proposal_id };

        let _res = execute(deps.as_mut(), mock_env(), owner_info, msg_end_vote).unwrap();



        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();

        assert_eq!(value.status, ProposalStatus::Passed);

    }

    #[test]
    fn end_vote_rejected() {
        let mut deps = mock_dependencies();

        const BASE: u128 = 1000;
        let msg = InstantiateMsg {
            denom: String::from(VOTING_TOKEN),
        };
        let owner_info = mock_info("creator", &coins(1*BASE, &msg.denom));
        let _res = instantiate(deps.as_mut(), mock_env(), owner_info.clone(), msg.clone()).unwrap();

        let voter_info = mock_info("anyone", &coins(1*BASE, &msg.denom));

        let yes_vote = false;
        let proposal_id = Uint128::from(1u128);
        let msg_execute = ExecuteMsg::Vote {
            proposal_id,
            yes_vote,
        };
        let _res = execute(deps.as_mut(), mock_env(), voter_info, msg_execute).unwrap();

        let msg_end_vote = ExecuteMsg::EndVote { proposal_id };

        let _res = execute(deps.as_mut(), mock_env(), owner_info, msg_end_vote).unwrap();



        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProposal {
                proposal_id: Uint128::from(1u128),
            },
        )
        .unwrap();
        let value: GetProposalResponse = from_binary(&res).unwrap();

        assert_eq!(value.status, ProposalStatus::Rejected);

    }

    
}
