use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};

use crate::transaction::Transaction;

/// A struct that deals with expense tracking
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseTracker {
    pub valid_categories: BTreeSet<String>,
    pub valid_subcategories: BTreeMap<String, BTreeSet<String>>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub transactions: Vec<Transaction>,
}

impl ExpenseTracker {
    pub fn new() -> Self {
        ExpenseTracker {
            valid_categories: BTreeSet::new(),
            valid_subcategories: BTreeMap::new(),
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
            .or_insert(BTreeSet::new());
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

    // Load transactions from a CSV and generate an expense tracker
    pub fn load_transactions_from_file(
        &mut self,
        file_path: &str,
        generate_categories_and_sub: bool,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Open the CSV file
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error while loading the CSV of transactions.");
                return Err(e.into());
            }
        };

        // Create a CSV reader
        let mut rdr = csv::Reader::from_reader(file);

        // Read the header record (first line)
        if let Ok(header) = rdr.headers() {
            // Convert the header entries into a Vec<String>
            let header_list: Vec<String> = header.iter().map(|entry| entry.to_string()).collect();

            // Print the resulting Vec<String>
            println!("{:?}", header_list);
        } else {
            eprintln!("CSV file does not have a header row.");
        }

        let mut n_ignored_transactions: i32 = 0;

        // Iterate over each record in the CSV file
        for record in rdr.records() {
            // Handle each CSV record
            let csv_row = record?;

            let transaction = Transaction::from_csv_row(csv_row)?;

            if generate_categories_and_sub {
                self.add_category(&transaction.category);

                if let Some(transaction_subcategory) = &transaction.subcategory {
                    // We can directly unwrap this function because we just made the category of the
                    // transaction valid, meaning that this should not error
                    self.add_subcategory(&transaction.category, transaction_subcategory)
                        .unwrap();
                }
            }

            match self.add_transaction(transaction) {
                Ok(()) => (),
                Err(e) => {
                    if verbose {
                        println!("{}", e);
                    }
                    n_ignored_transactions += 1;
                }
            };
        }

        if verbose {
            println!(
                "Number of valid transactions extracted from the CSV: {}",
                self.transactions.len()
            );
            println!("Number of transactions ignored: {}", n_ignored_transactions);
        }

        Ok(())
    }

    // Load categories and sub-categories from a file
    pub fn load_info_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let expense_tracker: ExpenseTracker = serde_json::from_reader(reader)?;
        Ok(expense_tracker)
    }

    // Load categories and sub-categories from the transactions part of the expense tracker
    pub fn load_info_from_transactions(&mut self) {
        let cloned_transactions = self.transactions.clone();
        for transaction in cloned_transactions {
            self.add_category(&transaction.category);
            if let Some(transaction_subcategory) = &transaction.subcategory {
                // We can directly unwrap this function because we just made the category of the
                // transaction valid, meaning that this should not error
                self.add_subcategory(&transaction.category, transaction_subcategory)
                    .unwrap();
            }
        }
    }

    // Save categories and sub-categories to a file
    pub fn save_info_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
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
