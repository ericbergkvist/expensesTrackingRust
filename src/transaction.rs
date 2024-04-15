use chrono::NaiveDate;
use core::f32;
use csv::StringRecord;
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
            category_name: category.to_string(),
            subcategory_name: formatted_subcategory,
            tag: formatted_tag,
            note: formatted_note,
        };

        Ok(transaction)
    }

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
    fn from_name(name: &str) -> Category {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn read_empty_line() {
        let empty_line = StringRecord::new();
        Transaction::from_csv_row(empty_line).unwrap();
    }
}
