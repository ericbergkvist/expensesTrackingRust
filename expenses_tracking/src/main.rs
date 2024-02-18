use chrono::NaiveDate;
use core::f32;
use csv::StringRecord;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;

#[derive(Debug, PartialEq, Clone)]
struct Transaction {
    date: NaiveDate,
    amount: f32,
    category: String,
    subcategory: Option<String>,
    tag: Option<String>,
    note: Option<String>,
}

impl Transaction {
    fn new() -> Transaction {
        Transaction {
            date: NaiveDate::default(),
            amount: 0.,
            category: String::new(),
            subcategory: None,
            tag: None,
            note: None,
        }
    }

    // Creates a transaction from a CSV row
    fn from_csv_row(csv_row: StringRecord) -> Result<Transaction, Box<dyn Error>> {
        // Read all the relevant values in the CSV line
        let date = csv_row.get(0).ok_or("Date not found in the record")?;
        let amount_out = csv_row.get(1).ok_or("Amount out not found in the record")?;
        let amount_in = csv_row.get(2).ok_or("Amount in not found in the record")?;
        let category = csv_row.get(3).ok_or("Category not found in the record")?;
        let subcategory = csv_row.get(4);
        let tag = csv_row.get(5);
        let note = csv_row.get(6);

        let formatted_date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?;
        let parsed_amount_in = parse_amount(amount_in)?;
        let parsed_amount_out = parse_amount(amount_out)?;

        // Turn optional fields from Option<&str> to Option<String>
        let formatted_subcategory = subcategory.map(|s| s.to_string());
        let formatted_tag = tag.map(|s| s.to_string());
        let formatted_note = note.map(|s| s.to_string());

        let transaction = Transaction {
            date: formatted_date,
            amount: parsed_amount_in - parsed_amount_out,
            category: category.to_string(),
            subcategory: formatted_subcategory,
            tag: formatted_tag,
            note: formatted_note,
        };

        Ok(transaction)
    }

    /// Checks if the transaction's category is part of a set of valid categories
    fn is_category_valid(&self, valid_categories: &HashSet<String>) -> bool {
        valid_categories.contains(&self.category)
    }

    /// Checks if the transaction's sub-category is part of valid sub-categories
    fn is_subcategory_valid(&self, valid_subcategories: &HashMap<String, HashSet<String>>) -> bool {
        match &self.subcategory {
            None => {
                // The None sub-category is valid as long as its associated category doesn't have
                // sub-categories (and the associated category is valid, which needs to be checked
                // separately)
                !valid_subcategories.contains_key(&self.category)
            }
            Some(subcategory) => {
                // The sub-category is valid as long as it's associated with its category in the
                // set of valid sub-categories
                valid_subcategories
                    .get(&self.category)
                    .map_or(false, |subcategories| subcategories.contains(subcategory))
            }
        }
    }
}

#[derive(Debug)]
struct ExpenseTracker {
    valid_categories: HashSet<String>,
    valid_subcategories: HashMap<String, HashSet<String>>,
    transactions: Vec<Transaction>,
}

impl ExpenseTracker {
    fn new() -> Self {
        ExpenseTracker {
            valid_categories: HashSet::new(),
            valid_subcategories: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    fn add_category(&mut self, transaction_category: &str) {
        self.valid_categories
            .insert(transaction_category.to_string());
    }

    fn add_subcategory(&mut self, transaction_category: &str, transaction_subcategory: &str) {
        let subcategories = self
            .valid_subcategories
            .entry(transaction_category.to_string())
            .or_insert(HashSet::new());
        subcategories.insert(transaction_subcategory.to_string());
    }

    /// Adds a given transaction to the expense tracker if required conditions are met
    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
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

fn parse_amount(amount: &str) -> Result<f32, Box<dyn Error>> {
    let mut numeric_amount = 0.0;
    // If an amount has no value, we set it to zero
    if !amount.is_empty() {
        // The ' character is used to delimit thousands from hundreds in CHF, so we remove it if
        // present
        let formatted_amount = amount.replace('\'', "");
        numeric_amount = formatted_amount.parse()?;
    }

    Ok(numeric_amount)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Specify the path to your CSV file
    //let file_path = "/home/ericbergkvist/personal/expensesTrackingRust/transactions.csv";
    let file_path = "/Users/eric/Desktop/transactions.csv";

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

    let mut n_lines: i32 = 0;
    let mut n_ignored_transactions: i32 = 0;

    let mut expense_tracker = ExpenseTracker::new();
    expense_tracker.add_category("Nourriture");
    expense_tracker.add_subcategory("Nourriture", "Courses");
    expense_tracker.add_subcategory("Nourriture", "Restaurant");

    // Iterate over each record in the CSV file
    for record in rdr.records() {
        // Handle each CSV record
        let csv_row = record?;

        let transaction = Transaction::from_csv_row(csv_row)?;

        match expense_tracker.add_transaction(transaction) {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e);
                n_ignored_transactions += 1;
            }
        };

        n_lines += 1;
    }

    let sum_transactions: f32 = expense_tracker
        .transactions
        .iter()
        .map(|transaction| transaction.amount)
        .sum();
    println!("Sum of all transaction: {} CHF", sum_transactions);

    println!(
        "Number of lines in the CSV (excluding the header): {}",
        n_lines
    );
    println!(
        "Number of valid transactions extracted from the CSV: {}",
        expense_tracker.transactions.len()
    );
    println!("Number of transactions ignored: {}", n_ignored_transactions);

    println!("{:?}", expense_tracker);

    Ok(())
}

#[cfg(test)]
mod tests {
    // Import everything from the parent module
    use super::*;

    // TODO: add tests for transaction parsing from CSV
    // TODO: add helper functions to avoid duplicating code

    #[test]
    fn add_valid_transactions() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker.add_subcategory("Nourriture", "Courses");
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
        expense_tracker.add_subcategory("Nourriture", "Courses");
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
        expense_tracker.add_subcategory("Nourriture", "Courses");
        let mut transaction: Transaction = Transaction::new();
        transaction.category = String::from("Nourriture");
        expense_tracker.add_transaction(transaction).unwrap();
    }
}
