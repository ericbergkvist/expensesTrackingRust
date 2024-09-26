use chrono::NaiveDate;
use log::{debug, info, trace};
use std::error::Error;
use std::{collections::BTreeSet, path::PathBuf};

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};

use crate::transaction::{AsSubCategory, Category, SubCategory, Transaction, TransactionCsv};

/// A struct that deals with expense tracking.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExpenseTracker {
    pub valid_categories: BTreeSet<Category>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub transactions: Vec<Transaction>,
}

impl ExpenseTracker {
    /// Creates a new `ExpenseTracker` object.
    pub fn new() -> Self {
        ExpenseTracker {
            valid_categories: BTreeSet::new(),
            transactions: Vec::new(),
        }
    }

    /// Returns an `Option` which contains a reference to a `Category` if it matches the argument.
    pub fn get_category(&self, category_name: &str) -> Option<&Category> {
        self.valid_categories
            .iter()
            .find(|category| category.name == category_name.to_lowercase())
    }

    /// Returns an `Option` which contains a reference to a `SubCategory` if it matches the argument.
    pub fn get_subcategory(
        &self,
        subcategory_name: &str,
        category_name: &str,
    ) -> Option<&SubCategory> {
        match self.get_category(category_name) {
            Some(category) => category
                .subcategories
                .iter()
                .find(|subcategory| subcategory.name == subcategory_name.to_lowercase()),
            None => None,
        }
    }

    /// Adds a valid category if it doesn't exist yet.
    pub fn add_category(&mut self, category_name: &str, date_creation: Option<NaiveDate>) -> bool {
        // Check whether a category with the same name exists (case insensitive)
        if let Some(found_category) = self.get_category(category_name) {
            debug!(
                "Cannot add Category as one with the same name already exists: {:?}",
                found_category
            );
            return false;
        }

        let category_date: NaiveDate = match date_creation {
            Some(date) => date,
            // If no date was used as an input, use today's date
            None => chrono::prelude::Local::now().naive_local().into(),
        };

        let new_category = Category {
            // All category names are lower case to avoid any confusion
            name: category_name.to_lowercase(),
            date_added: category_date,
            subcategories: BTreeSet::new(),
        };
        self.valid_categories.insert(new_category)
    }

    /// Adds a valid sub-category associated with a category if it doesn't exist yet and if the
    /// category is valid.
    pub fn add_subcategory(
        &mut self,
        category_name: &str,
        subcategory_name: &str,
        date_creation: Option<NaiveDate>,
    ) -> Result<(), Box<dyn Error>> {
        // We will need to clone the category in some cases
        let cloned_category: Category;

        match self.get_category(category_name) {
            Some(category) => cloned_category = category.clone(),
            None => {
                return Err("Sub-category cannot be added because its category is invalid".into())
            }
        }

        if self
            .get_subcategory(subcategory_name, category_name)
            .is_some()
        {
            return Err("The subcategory name already exists".into());
        }

        // We are in the situation where the sub-category needs to be added to the category.
        // As the elements in a `BTreeSet` can by default not be modified, we need to remove
        // the `Category` object, modify it, and insert it again

        // Safe to unwrap because the category exists if the code arrives here
        let mut extracted_category = self.valid_categories.take(&cloned_category).unwrap();

        let subcategory_date: NaiveDate = match date_creation {
            Some(date) => date,
            // If no date was used as an input, use today's date
            None => chrono::prelude::Local::now().naive_local().into(),
        };

        let new_subcategory = SubCategory {
            name: subcategory_name.to_lowercase(),
            date_added: subcategory_date,
        };

        extracted_category.subcategories.insert(new_subcategory);

        self.valid_categories.insert(extracted_category);

        Ok(())
    }

    /// Checks if a transaction is valid.
    pub fn is_transaction_valid(&self, transaction: &Transaction) -> Result<(), Box<dyn Error>> {
        let maybe_category = self.get_category(&transaction.category_name);
        match maybe_category {
            None => Err("Invalid category in transaction".into()),
            Some(category) => {
                match &transaction.subcategory_name {
                    None => {
                        // The None sub-category is valid as long as its associated category doesn't
                        // have sub-categories
                        if category.subcategories.is_empty() {
                            return Ok(());
                        }
                        Err(
                            "No sub-category set in transaction although the category has some"
                                .into(),
                        )
                    }
                    Some(subcategory_name) => {
                        // The sub-category is valid as long as it's associated with its category
                        // in the set of valid sub-categories
                        if category
                            .subcategories
                            .contains(&subcategory_name.as_subcategory())
                        {
                            return Ok(());
                        }

                        if category
                            .subcategories
                            .iter()
                            .any(|subcategory| subcategory.name == subcategory_name.to_lowercase())
                        {
                            return Ok(());
                        }

                        Err("Sub-category set in transaction does not exist in category".into())
                    }
                }
            }
        }
    }

    /// Adds a given transaction to the expense tracker if required conditions are met.
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        // Only add the transaction if its category is valid
        self.is_transaction_valid(&transaction)?;

        self.transactions.push(transaction);
        Ok(())
    }

    /// Load transactions from a CSV and generate an expense tracker.
    pub fn load_transactions_from_file(
        &mut self,
        file_path: &PathBuf,
        generate_categories_and_sub: bool,
    ) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(file_path)
            .map_err(|e| format!("Failed to load the CSV of transactions: {e}"))?;

        let mut n_ignored_transactions: i32 = 0;

        // Iterate over each record in the CSV file
        for record in rdr.deserialize() {
            let transaction_csv: TransactionCsv =
                record.map_err(|e| format!("Failed to deserialize the CSV transaction: {e}"))?;
            let transaction = Transaction::try_from(transaction_csv)
                .map_err(|e| format!("Failed to convert CSV transaction to transaction: {e}"))?;

            if generate_categories_and_sub {
                self.add_category(&transaction.category_name, Some(transaction.date));

                if let Some(transaction_subcategory) = &transaction.subcategory_name {
                    match self.add_subcategory(
                        &transaction.category_name,
                        transaction_subcategory,
                        Some(transaction.date),
                    ) {
                        Ok(()) => (),
                        Err(e) => debug!("{}", e),
                    };
                }
            }

            match self.add_transaction(transaction) {
                Ok(()) => (),
                Err(e) => {
                    trace!("{}", e);
                    n_ignored_transactions += 1;
                }
            };
        }

        info!(
            "Number of valid transactions extracted from the CSV: {}",
            self.transactions.len()
        );
        info!("Number of transactions ignored: {}", n_ignored_transactions);

        Ok(())
    }

    pub fn write_transactions_to_file(&self, output_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(output_path)
            .map_err(|e| format!("Failed to open output CSV file: {e}"))?;

        writer
            .write_record([
                "date",
                "amount_out",
                "amount_in",
                "category",
                "subcategory",
                "tag",
                "note",
            ])
            .map_err(|e| format!("Failed to write header to output CSV file: {e}"))?;

        for transaction in &self.transactions {
            let amount_out = if transaction.amount < 0.0 {
                format!("{:.2}", -transaction.amount)
            } else {
                String::new()
            };
            let amount_in = if transaction.amount >= 0.0 {
                format!("{:.2}", transaction.amount)
            } else {
                String::new()
            };

            writer
                .write_record(&[
                    transaction.date.format("%d.%m.%Y").to_string(),
                    amount_out,
                    amount_in,
                    transaction.category_name.clone(),
                    transaction
                        .subcategory_name
                        .clone()
                        .unwrap_or("".to_string()),
                    transaction.tag.clone().unwrap_or("".to_string()),
                    transaction.note.clone().unwrap_or("".to_string()),
                ])
                .map_err(|e| format!("Failed to write a transaction to output CSV file: {e}"))?;
        }

        // Flush the writer to make sure all data is written to the file
        writer
            .flush()
            .map_err(|e| format!("Failed to flush output CSV file: {e}"))?;

        Ok(())
    }

    /// Loads categories and sub-categories from a file.
    pub fn load_info_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let expense_tracker: ExpenseTracker = serde_json::from_reader(reader)?;
        Ok(expense_tracker)
    }

    /// Loads categories and sub-categories from the transactions part of the expense tracker.
    pub fn load_info_from_transactions(&mut self) {
        let cloned_transactions = self.transactions.clone();
        for transaction in cloned_transactions {
            self.add_category(&transaction.category_name, Some(transaction.date));
            if let Some(transaction_subcategory) = &transaction.subcategory_name {
                // We can directly unwrap this function because we just made the category of the
                // transaction valid, meaning that this should not error
                self.add_subcategory(
                    &transaction.category_name,
                    transaction_subcategory,
                    Some(transaction.date),
                )
                .unwrap();
            }
        }
    }

    /// Save categories and sub-categories to a file.
    pub fn save_info_to_file(&self, file_path: PathBuf) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)
            .map_err(|e| format!("Failed to create config file: {e}"))?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .map_err(|e| format!("Failed to write categories to config: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Import everything from the parent module
    use super::*;
    use chrono::NaiveDate;

    impl Default for Transaction {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Transaction {
        pub fn new() -> Transaction {
            Transaction {
                date: NaiveDate::default(),
                amount: 0.,
                category_name: String::new(),
                subcategory_name: None,
                tag: None,
                note: None,
            }
        }
    }

    #[test]
    #[should_panic(expected = "Sub-category cannot be added because its category is invalid")]
    fn add_subcategory_invalid_category() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", None);
        expense_tracker
            .add_subcategory("Transports", "Train", None)
            .unwrap();
    }

    #[test]
    fn add_category() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", Some(NaiveDate::default()));
        let category = Category {
            name: "Nourriture".to_lowercase(),
            date_added: NaiveDate::default(),
            subcategories: BTreeSet::new(),
        };
        assert_eq!(
            expense_tracker.valid_categories.pop_first().unwrap(),
            category
        );
    }

    #[test]
    fn add_subcategory_and_check_category() {}

    #[test]
    fn add_subcategory_valid() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", None);
        expense_tracker
            .add_subcategory("Nourriture", "Courses", None)
            .unwrap();

        let mut category = Category {
            name: "Nourriture".to_lowercase(),
            date_added: NaiveDate::default(),
            subcategories: BTreeSet::new(),
        };
        let subcategory = SubCategory {
            name: "Courses".to_lowercase(),
            date_added: NaiveDate::default(),
        };
        category.subcategories.insert(subcategory);

        assert_eq!(
            expense_tracker.valid_categories.pop_first().unwrap(),
            category
        );
    }

    #[test]
    fn add_transactions_valid() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", Some(NaiveDate::default()));
        expense_tracker
            .add_subcategory("Nourriture", "Courses", Some(NaiveDate::default()))
            .unwrap();

        expense_tracker.add_category("Transports", None);

        let mut transactions: Vec<Transaction> = Vec::new();

        let mut transaction_1: Transaction = Transaction::new();
        transaction_1.category_name = "Nourriture".to_lowercase();
        transaction_1.subcategory_name = Some("Courses".to_lowercase());
        transactions.push(transaction_1);

        let mut transaction_2: Transaction = Transaction::new();
        transaction_2.category_name = "Transports".to_lowercase();
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
        expense_tracker.add_category("Nourriture", None);
        let mut valid_transaction: Transaction = Transaction::new();
        valid_transaction.category_name = String::from("Transports");
        expense_tracker.add_transaction(valid_transaction).unwrap();
    }

    #[test]
    #[should_panic(expected = "Sub-category set in transaction does not exist in category")]
    fn add_transaction_invalid_subcategory() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", None);
        expense_tracker
            .add_subcategory("Nourriture", "Courses", None)
            .unwrap();
        let mut transaction: Transaction = Transaction::new();
        transaction.category_name = "Nourriture".to_lowercase();
        transaction.subcategory_name = Some("Restaurant".to_lowercase());
        expense_tracker.add_transaction(transaction).unwrap();
    }

    #[test]
    #[should_panic(expected = "No sub-category set in transaction although the category has some")]
    fn add_transaction_missing_subcategory() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture", None);
        expense_tracker
            .add_subcategory("Nourriture", "Courses", None)
            .unwrap();
        let mut transaction: Transaction = Transaction::new();
        transaction.category_name = "Nourriture".to_lowercase();
        expense_tracker.add_transaction(transaction).unwrap();
    }

    #[test]
    fn load_transactions_from_file_and_write_to_another() {
        extern crate tempdir;
        use std::str::FromStr;
        use tempdir::TempDir;
        let input_path = PathBuf::from_str("test_data/transactions_example.csv").unwrap();
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker
            .load_transactions_from_file(&input_path, true)
            .unwrap();
        let tmp_dir = TempDir::new("example").unwrap();
        let output_path = tmp_dir.path().join("transactions_out.csv");
        expense_tracker
            .write_transactions_to_file(&output_path)
            .unwrap();

        let input_data = csv_to_vec(&input_path).unwrap();
        let output_data = csv_to_vec(&output_path).unwrap();
        assert_eq!(input_data, output_data);

        drop(output_path);
        tmp_dir.close().unwrap();
    }

    fn csv_to_vec(csv_path: &PathBuf) -> Result<Vec<csv::StringRecord>, csv::Error> {
        let mut reader = csv::ReaderBuilder::new().from_path(csv_path)?;
        reader.records().collect::<Result<Vec<_>, _>>()
    }
}
