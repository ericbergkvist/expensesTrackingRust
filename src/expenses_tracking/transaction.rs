use chrono::NaiveDate;
use core::f32;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;

/// A struct that represents a transaction
#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: f32,
    pub category_name: String,
    pub subcategory_name: Option<String>,
    pub tag: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionCsv {
    date: String,
    amount_out: String,
    amount_in: String,
    category: String,
    subcategory: String,
    tag: String,
    note: String,
}

impl TryFrom<TransactionCsv> for Transaction {
    type Error = Box<dyn Error>;

    fn try_from(transaction_csv: TransactionCsv) -> Result<Self, Self::Error> {
        let formatted_date =
            chrono::NaiveDate::parse_from_str(&transaction_csv.date, "%d.%m.%Y")
                .map_err(|e| format!("Failed to parse date from CSV transaction: {e}"))?;
        let parsed_amount_in = parse_amount(&transaction_csv.amount_in)
            .map_err(|e| format!("Failed to parse amount_in from CSV transaction: {e}"))?;
        let parsed_amount_out = parse_amount(&transaction_csv.amount_out)
            .map_err(|e| format!("Failed to parse amount_out from CSV transaction: {e}"))?;

        let transaction = Transaction {
            date: formatted_date,
            amount: parsed_amount_in - parsed_amount_out,
            category_name: transaction_csv.category,
            subcategory_name: string_to_option(transaction_csv.subcategory),
            tag: string_to_option(transaction_csv.tag),
            note: string_to_option(transaction_csv.note),
        };

        Ok(transaction)
    }
}

impl Transaction {
    fn to_csv_row() {
        todo!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Category {
    pub name: String,
    pub subcategories: BTreeSet<SubCategory>,
    pub date_added: NaiveDate,
}

// For convenience, we compare `Category` simply by their name
impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Category {}

impl Category {
    /// Creates a default `Category` object from a name.
    pub fn from_name(name: &str) -> Category {
        Category {
            name: name.to_lowercase(),
            date_added: NaiveDate::default(),
            subcategories: BTreeSet::new(),
        }
    }
}
pub trait AsCategory {
    fn as_category(self) -> Category;
}

// For convenience we add a trait to `&str` objects such that they can be used to create default
// `Category` objects easily
impl AsCategory for &str {
    fn as_category(self) -> Category {
        Category::from_name(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct SubCategory {
    pub name: String,
    pub date_added: NaiveDate,
}

impl PartialEq for SubCategory {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for SubCategory {}

impl SubCategory {
    /// Creates a default `SubCategory` object from a name.
    fn from_name(name: &str) -> SubCategory {
        SubCategory {
            name: name.to_lowercase(),
            date_added: NaiveDate::default(),
        }
    }
}

pub trait AsSubCategory {
    fn as_subcategory(self) -> SubCategory;
}

impl AsSubCategory for &str {
    fn as_subcategory(self) -> SubCategory {
        SubCategory::from_name(self)
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

fn string_to_option(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    #[should_panic]
    fn read_empty_line() {
        let empty_line = StringRecord::new();
        let _transaction_csv: TransactionCsv = empty_line.deserialize(None).unwrap();
    }
}
