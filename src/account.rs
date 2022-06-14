use crate::constants::PRECISION;

/// Struct to hold data and methods for an account
#[derive(Debug, PartialEq)]
pub struct Account {
    /// Assuming 1 account per client for simplicity
    pub id: u16,

    /// Funds which are available for withdrawal by client
    pub available: f64,

    /// Amount held due to disputes
    pub held: f64,

    /// Status of account, determined by txn behavior
    pub frozen: bool,
}

impl Account {
    pub fn get_total(&self) -> f64 {
        self.available + self.held
    }

    // dop
    pub fn get_display_str(&self) -> String {
        format!(
            "{:?},{:.*},{:.*},{:.*},{:?}",
            self.id,
            PRECISION,
            self.available,
            PRECISION,
            self.held,
            PRECISION,
            self.get_total(),
            self.frozen
        )
    }

    pub fn print_std_out(&self) {
        println!("{}", self.get_display_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::account::Account;

    #[test]
    fn tst_get_total() {
        let accnt = Account {
            id: 1,
            available: 10.0,
            held: 5.0,
            frozen: false,
        };
        assert_eq!(accnt.get_total(), 15.0);
    }

    #[test]
    fn tst_print_std_out() {
        let accnt = Account {
            id: 1,
            available: 10.0,
            held: 5.0,
            frozen: false,
        };
        assert_eq!(accnt.get_display_str(), "1,10.0000,5.0000,15.0000,false");
    }
}
