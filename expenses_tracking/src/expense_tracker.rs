use std::collections::{HashMap, HashSet};
use std::error::Error;

use crate::transaction::Transaction;

/// A struct that deals with expense tracking
#[derive(Debug)]
pub struct ExpenseTracker {
    pub valid_categories: HashSet<String>,
    pub valid_subcategories: HashMap<String, HashSet<String>>,
    pub transactions: Vec<Transaction>,
}

impl ExpenseTracker {
    pub fn new() -> Self {
        ExpenseTracker {
            valid_categories: HashSet::new(),
            valid_subcategories: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    /// Adds a valid category if it doesn't exist yet
    pub fn add_category(&mut self, transaction_category: &str) {
        self.valid_categories
            .insert(transaction_category.to_string());
    }

    /// Adds a valid sub-category associated with a category if it doesn't exist yet and if the
    /// category is valid
    pub fn add_subcategory(
        &mut self,
        transaction_category: &str,
        transaction_subcategory: &str,
    ) -> Result<(), Box<dyn Error>> {
        if !self.valid_categories.contains(transaction_category) {
            return Err("Sub-category cannot be added because its category is invalid".into());
        }
        let subcategories = self
            .valid_subcategories
            .entry(transaction_category.to_string())
            .or_insert(HashSet::new());
        subcategories.insert(transaction_subcategory.to_string());

        Ok(())
    }

    /// Adds a given transaction to the expense tracker if required conditions are met
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        // Only add the transaction if its category is valid
        if !transaction.is_category_valid(&self.valid_categories) {
            return Err("Invalid category".into());
        }

        // Only add the transaction if its sub-category is valid
        if !transaction.is_subcategory_valid(&self.valid_subcategories) {
            return Err("Invalid sub-category (not linked to category)".into());
        }

        self.transactions.push(transaction);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Import everything from the parent module
    use super::*;
    use chrono::NaiveDate;

    impl Transaction {
        pub fn new() -> Transaction {
            Transaction {
                date: NaiveDate::default(),
                amount: 0.,
                category: String::new(),
                subcategory: None,
                tag: None,
                note: None,
            }
        }
    }

    // TODO: add tests for transaction parsing from CSV
    // TODO: add helper functions to avoid duplicating code

    #[test]
    #[should_panic(expected = "Sub-category cannot be added because its category is invalid")]
    fn add_subcategory_invalid_category() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker
            .add_subcategory("Transports", "Train")
            .unwrap();
    }

    #[test]
    fn add_transactions_valid() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker
            .add_subcategory("Nourriture", "Courses")
            .unwrap();
        expense_tracker.add_category("Transports");

        let mut transactions: Vec<Transaction> = Vec::new();

        let mut transaction_1: Transaction = Transaction::new();
        transaction_1.category = String::from("Nourriture");
        transaction_1.subcategory = Some(String::from("Courses"));
        transactions.push(transaction_1);

        let mut transaction_2: Transaction = Transaction::new();
        transaction_2.category = String::from("Transports");
        transactions.push(transaction_2);

        // Adding a transaction to the expense tracker takes ownership of the Transaction object,
        // so we need to clone transactions before using add_transaction() to be able to check the
        // equality
        let cloned_transactions = transactions.clone();
        for transaction in transactions {
            expense_tracker.add_transaction(transaction).unwrap();
        }
        assert_eq!(expense_tracker.transactions, cloned_transactions);
    }

    #[test]
    #[should_panic(expected = "Invalid category")]
    fn add_transaction_invalid_category() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        let mut valid_transaction: Transaction = Transaction::new();
        valid_transaction.category = String::from("Transports");
        expense_tracker.add_transaction(valid_transaction).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid sub-category")]
    fn add_transaction_invalid_subcategory() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker
            .add_subcategory("Nourriture", "Courses")
            .unwrap();
        let mut transaction: Transaction = Transaction::new();
        transaction.category = String::from("Nourriture");
        transaction.subcategory = Some(String::from("Restaurant"));
        expense_tracker.add_transaction(transaction).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid sub-category")]
    fn add_transaction_missing_subcategory() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker
            .add_subcategory("Nourriture", "Courses")
            .unwrap();
        let mut transaction: Transaction = Transaction::new();
        transaction.category = String::from("Nourriture");
        expense_tracker.add_transaction(transaction).unwrap();
    }
}
