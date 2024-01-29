use chrono::NaiveDate;
use core::f32;
use csv::StringRecord;
use std::error::Error;
use std::fs::File;

enum TransactionCategory {
    Nourriture(NourritureTypes),
    Transports(TransportsTypes),
}

enum NourritureTypes {
    Lunch,
    Courses,
}

enum TransportsTypes {
    Train,
    ZurichPublicTransport,
}

struct Transaction {
    date: NaiveDate,
    amount: f32,
    category: TransactionCategory,
    name: String,
    note: String,
}

impl Transaction {
    fn default() -> Transaction {
        Transaction {
            date: NaiveDate::default(),
            amount: 0.,
            category: TransactionCategory::Nourriture(NourritureTypes::Lunch),
            name: "".to_string(),
            note: "".to_string(),
        }
    }
}

fn parse_transaction(csv_line: StringRecord) -> Result<Transaction, Box<dyn Error>> {
    // Create a default transaction
    let mut transaction = Transaction::default();

    // Read all the relevant values in the CSV line
    let date_str = csv_line.get(0).ok_or("Date not found in the record")?;
    let mut amount_out_str: String = csv_line
        .get(1)
        .ok_or("Amount out not found in the record")?
        .to_string();
    let mut amount_in_str: String = csv_line
        .get(2)
        .ok_or("Amount in not found in the record")?
        .to_string();
    let category_str = csv_line.get(3).ok_or("Category not found in the record")?;
    let type_str = csv_line.get(4).ok_or("Type not found in the record")?;
    let name_str = csv_line.get(5).ok_or("Name not found in the record")?;
    let note_str = csv_line.get(6).ok_or("Note not found in the record")?;

    // Parse the strings
    let date = chrono::NaiveDate::parse_from_str(date_str, "%d.%m.%Y")?;

    let mut amount_out: f32 = 0.0;
    let mut amount_in: f32 = 0.0;

    if !amount_out_str.is_empty() {
        if amount_out_str.contains('\'') {
            amount_out_str = amount_out_str.replace('\'', "");
        }
        amount_out = amount_out_str.parse()?;
    }
    if !amount_in_str.is_empty() {
        if amount_in_str.contains('\'') {
            amount_in_str = amount_in_str.replace('\'', "");
        }
        amount_in = amount_in_str.parse()?;
    }

    // Create a Transaction entry and return it using a Result() to propagate error handling to the
    // main()
    transaction.date = date;
    transaction.amount = amount_in - amount_out;
    Ok(transaction)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Specify the path to your CSV file
    let file_path = "/Users/eric/Desktop/transactions.csv";

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

    let mut transactions: Vec<Transaction> = Vec::new();
    let mut n_lines: i32 = 0;

    // Iterate over each record in the CSV file
    for result in rdr.records() {
        // Handle each CSV record
        let csv_line = result?;

        // Only push valid transactions to the list of transactions
        if let Ok(transaction) = parse_transaction(csv_line) {
            transactions.push(transaction);
        }

        /*
        // The date is in the first column
        if let Some(date_str) = transaction.get(0) {
            // Parse the date using chrono
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%d.%m.%Y") {
                // Use the parsed date as needed
                println!("Parsed date: {:?}", date);
            } else {
                eprintln!("Error parsing date: {}", date_str);
            }
        } else {
            eprintln!("No date found in the record");
        }
        */
        n_lines += 1;
    }

    let sum_transactions: f32 = transactions
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
        transactions.len()
    );

    Ok(())
}
