use crate::bid::Bid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    pub winner: String,
    pub clearing_price: u64,
}

pub fn settle_vickrey(bids: &[Bid]) -> Option<Settlement> {
    if bids.is_empty() {
        return None;
    }

    let mut sorted = bids.to_vec();
    sorted.sort_by(|left, right| right.amount.cmp(&left.amount));

    let winner = sorted.first()?;
    let clearing_price = sorted.get(1).map(|bid| bid.amount).unwrap_or(winner.amount);

    Some(Settlement {
        winner: winner.bidder.clone(),
        clearing_price,
    })
}
