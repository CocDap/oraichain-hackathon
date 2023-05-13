use crate::error::ContractError;
use cosmwasm_std::Coin;

pub fn validate_sent_sufficient_coin(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), ContractError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        println!("require amount:{}", required_amount);
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == required_coin.denom && coin.amount.u128() == required_amount
            });
            
            return if sent_sufficient_funds {
                Ok(())
            } else {
                Err(ContractError::InsufficientFundsSent {})
            };
        }
    }
    Ok(())
}
