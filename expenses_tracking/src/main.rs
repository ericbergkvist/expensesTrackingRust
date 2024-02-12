use chrono::NaiveDate;
use core::f32;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;

#[derive(Debug)]
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

    fn add_transaction(
        &mut self,
        date: &str,
        amount_out: &str,
        amount_in: &str,
        category: &str,
        subcategory: &str,
        tag: &str,
        note: &str,
    ) -> Result<(), Box<dyn Error>> {
        if !self.valid_categories.contains(category) {
            return Err("Invalid category".into());
        }

        if !self
            .valid_subcategories
            .get(category)
            .map_or(false, |subcategories| subcategories.contains(subcategory))
        {
            return Err("Invalid sub-category (not linked to category)".into());
        }

        let transaction_date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?;

        let mut amount_out_numeric: f32 = 0.0;
        let mut amount_in_numeric: f32 = 0.0;

        if !amount_out.is_empty() {
            let mut amount_out_string: String = amount_out.to_string();
            if amount_out.contains('\'') {
                amount_out_string = amount_out.replace('\'', "");
            }
            amount_out_numeric = amount_out_string.parse()?;
        }
        if !amount_in.is_empty() {
            let mut amount_in_string: String = amount_in.to_string();
            if amount_in.contains('\'') {
                amount_in_string = amount_in.replace('\'', "");
            }
            amount_in_numeric = amount_in_string.parse()?;
        }

        let transaction = Transaction {
            date: transaction_date,
            amount: amount_in_numeric - amount_out_numeric,
            category: category.to_string(),
            subcategory: subcategory.to_string(),
            tag: tag.to_string(),
            note: note.to_string(),
        };

        self.transactions.push(transaction);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Specify the path to your CSV file
    let file_path = "/home/ericbergkvist/personal/expensesTrackingRust/transactions.csv";

    // Open the CSV file
    // If it errors, the main will directly return an Err that is handled by the fact that the main
    // returns a Result as well.
    let file = File::open(file_path)?;

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
    for result in rdr.records() {
        // Handle each CSV record
        let csv_line = result?;

        // Read all the relevant values in the CSV line
        let date_str = csv_line.get(0).ok_or("Date not found in the record")?;
        let amount_out_str = csv_line
            .get(1)
            .ok_or("Amount out not found in the record")?;
        let amount_in_str = csv_line.get(2).ok_or("Amount in not found in the record")?;
        let category_str = csv_line.get(3).ok_or("Category not found in the record")?;
        let subcategory_str = csv_line
            .get(4)
            .ok_or("Sub-category not found in the record")?;
        let tag_str = csv_line.get(5).ok_or("Tag not found in the record")?;
        let note_str = csv_line.get(6).ok_or("Note not found in the record")?;

        match expense_tracker.add_transaction(
            date_str,
            amount_out_str,
            amount_in_str,
            category_str,
            subcategory_str,
            tag_str,
            note_str,
        ) {
            Ok(_) => (),
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
