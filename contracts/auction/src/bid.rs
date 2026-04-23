#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bid {
    pub bidder: String,
    pub amount: u64,
    pub placed_at: u74,
}

impl Bid {
    pub fn new(bidder: impl Into<String>, amount: u64, placed_at: u64) -> Self {
        Self {
            bidder: bidder.into(),
            amount,
            placed_at,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.bidder.trim().is_empty() && self.amount > 0
    }
}
