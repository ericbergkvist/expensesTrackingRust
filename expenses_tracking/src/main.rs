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
    subcategory: String,
    tag: String,
    note: String,
}

impl Transaction {
    fn new() -> Transaction {
        Transaction {
            date: NaiveDate::default(),
            amount: 0.,
            category: String::new(),
            subcategory: String::new(),
            tag: String::new(),
            note: String::new(),
        }
    }

    fn from_csv_line(csv_line: StringRecord) -> Result<Transaction, Box<dyn Error>> {
        // Read all the relevant values in the CSV line
        let date = csv_line.get(0).ok_or("Date not found in the record")?;
        let amount_out = csv_line
            .get(1)
            .ok_or("Amount out not found in the record")?;
        let amount_in = csv_line.get(2).ok_or("Amount in not found in the record")?;
        let category = csv_line.get(3).ok_or("Category not found in the record")?;
        let subcategory = csv_line
            .get(4)
            .ok_or("Sub-category not found in the record")?;
        let tag = csv_line.get(5).ok_or("Tag not found in the record")?;
        let note = csv_line.get(6).ok_or("Note not found in the record")?;

        let formatted_date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?;
        let parsed_amount_in = parse_amount(amount_in)?;
        let parsed_amount_out = parse_amount(amount_out)?;

        let transaction = Transaction {
            date: formatted_date,
            amount: parsed_amount_in - parsed_amount_out,
            category: category.to_string(),
            subcategory: subcategory.to_string(),
            tag: tag.to_string(),
            note: note.to_string(),
        };

        Ok(transaction)
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

    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        if !self
            .valid_categories
            .contains(transaction.category.as_str())
        {
            return Err("Invalid category".into());
        }

        if !self
            .valid_subcategories
            .get(transaction.category.as_str())
            .map_or(false, |subcategories| {
                subcategories.contains(transaction.subcategory.as_str())
            })
        {
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
        let csv_line = record?;

        let transaction = Transaction::from_csv_line(csv_line)?;

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
    // Import everything needed from the parent module
    use super::*;

    // TODO: add tests for transaction parsing from CSV
    // TODO: add helper functions to avoid duplicating code

    #[test]
    fn add_valid_transaction() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker.add_subcategory("Nourriture", "Courses");
        let valid_transaction = Transaction {
            date: chrono::NaiveDate::parse_from_str("01.01.2024", "%d.%m.%Y").unwrap(),
            amount: -200.0,
            category: String::from("Nourriture"),
            subcategory: String::from("Courses"),
            tag: String::from(""),
            note: String::from(""),
        };
        // Adding the transaction to the expense tracker takes ownership of the Transaction object,
        // so we need to clone it before using add_transaction() to be able to check the equality
        let cloned_valid_transaction = valid_transaction.clone();
        expense_tracker.add_transaction(valid_transaction).unwrap();
        assert_eq!(expense_tracker.transactions[0], cloned_valid_transaction);
    }

    #[test]
    #[should_panic(expected = "Invalid category")]
    fn add_invalid_category() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        let invalid_transaction = Transaction {
            date: chrono::NaiveDate::parse_from_str("01.01.2024", "%d.%m.%Y").unwrap(),
            amount: -200.0,
            category: String::from("Transports"),
            subcategory: String::from(""),
            tag: String::from(""),
            note: String::from(""),
        };
        expense_tracker
            .add_transaction(invalid_transaction)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid sub-category")]
    fn add_invalid_subcategory() {
        let mut expense_tracker = ExpenseTracker::new();
        expense_tracker.add_category("Nourriture");
        expense_tracker.add_subcategory("Nourriture", "Courses");
        let invalid_transaction = Transaction {
            date: chrono::NaiveDate::parse_from_str("01.01.2024", "%d.%m.%Y").unwrap(),
            amount: -200.0,
            category: String::from("Nourriture"),
            subcategory: String::from("Restaurant"),
            tag: String::from(""),
            note: String::from(""),
        };
        expense_tracker
            .add_transaction(invalid_transaction)
            .unwrap();
    }
}
