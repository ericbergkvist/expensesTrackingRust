use chrono::NaiveDate;
use core::f32;
use csv::StringRecord;
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: f32,
    pub category: String,
    pub subcategory: Option<String>,
    pub tag: Option<String>,
    pub note: Option<String>,
}

impl Transaction {
    // Creates a transaction from a CSV row
    pub fn from_csv_row(csv_row: StringRecord) -> Result<Transaction, Box<dyn Error>> {
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
    pub fn is_category_valid(&self, valid_categories: &HashSet<String>) -> bool {
        valid_categories.contains(&self.category)
    }

    /// Checks if the transaction's sub-category is part of valid sub-categories
    pub fn is_subcategory_valid(
        &self,
        valid_subcategories: &HashMap<String, HashSet<String>>,
    ) -> bool {
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
