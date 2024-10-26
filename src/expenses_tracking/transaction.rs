use chrono::NaiveDate;
use core::f32;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::rc::Rc;

/// Represents a transaction, including a reference to a `Category`
#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: f32,
    pub category: Rc<Category>,
    pub subcategory_name: Option<String>,
    pub tag: Option<String>,
    pub note: Option<String>,
}

/// Version of a transaction containing only `String` objects, which makes it automatically
/// deserializable by the `csv` crate. Later converted to a `TransactionParsed` and finally a
/// `Transaction`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct TransactionCsv {
    date: String,
    amount_out: String,
    amount_in: String,
    category: String,
    subcategory: String,
    tag: String,
    note: String,
}

/// Intermediate state of a transaction before creating a `Transaction` by resolving references to
/// objects, e.g. `Category`.
pub struct TransactionParsed {
    pub date: NaiveDate,
    pub amount: f32,
    pub category: String,
    pub subcategory_name: Option<String>,
    pub tag: Option<String>,
    pub note: Option<String>,
}

impl Transaction {
    pub fn from(transaction_parsed: TransactionParsed, category: Rc<Category>) -> Transaction {
        Transaction {
            date: transaction_parsed.date,
            amount: transaction_parsed.amount,
            category,
            subcategory_name: transaction_parsed.subcategory_name,
            tag: transaction_parsed.tag,
            note: transaction_parsed.note,
        }
    }
}

impl TransactionParsed {
    /// Resolves the references to objects (e.g. `Category`) in a `TransactionParsed` to create a
    /// `Transaction`, if conditions are met.
    pub fn resolve_references(
        self,
        maybe_category: Option<Rc<Category>>,
    ) -> Result<Transaction, Box<dyn Error>> {
        match maybe_category {
            None => Err("Invalid category in transaction".into()),
            Some(category) => {
                match self.subcategory_name {
                    None => {
                        // The None sub-category is valid as long as its associated category doesn't
                        // have sub-categories
                        if category.subcategories.is_empty() {
                            return Ok(Transaction::from(self, category));
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
                            return Ok(Transaction::from(self, category));
                        }

                        // Is the code below redundant with what's above?
                        if category
                            .subcategories
                            .iter()
                            .any(|subcategory| subcategory.name == subcategory_name.to_lowercase())
                        {
                            return Ok(Transaction::from(self, category));
                        }

                        Err("Sub-category set in transaction does not exist in category".into())
                    }
                }
            }
        }
    }
}

impl TryFrom<TransactionCsv> for TransactionParsed {
    type Error = Box<dyn Error>;

    fn try_from(transaction_csv: TransactionCsv) -> Result<Self, Self::Error> {
        let formatted_date =
            chrono::NaiveDate::parse_from_str(&transaction_csv.date, "%d.%m.%Y")
                .map_err(|e| format!("Failed to parse date from CSV transaction: {e}"))?;
        let parsed_amount_in = parse_amount(&transaction_csv.amount_in)
            .map_err(|e| format!("Failed to parse amount_in from CSV transaction: {e}"))?;
        let parsed_amount_out = parse_amount(&transaction_csv.amount_out)
            .map_err(|e| format!("Failed to parse amount_out from CSV transaction: {e}"))?;

        let transaction_parsed = TransactionParsed {
            date: formatted_date,
            amount: parsed_amount_in - parsed_amount_out,
            category: transaction_csv.category,
            subcategory_name: string_to_option(transaction_csv.subcategory),
            tag: string_to_option(transaction_csv.tag),
            note: string_to_option(transaction_csv.note),
        };

        Ok(transaction_parsed)
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
    fn deserialize_empty_line() {
        let empty_line = StringRecord::new();
        let _transaction_csv: TransactionCsv = empty_line.deserialize(None).unwrap();
    }

    #[test]
    fn deserialize_transaction() {
        let transaction_csv = TransactionCsv {
            date: "01.01.1970".to_string(),
            amount_out: "30".to_string(),
            amount_in: "".to_string(),
            category: "Food".to_string(),
            subcategory: "".to_string(),
            tag: "Invited others".to_string(),
            note: "This is a note".to_string(),
        };

        // Note that the date is the order of keys is on purpose not the same as in TransactionCsv
        let header = StringRecord::from(vec![
            "amount_out",
            "amount_in",
            "category",
            "subcategory",
            "tag",
            "date",
            "note",
        ]);
        let transaction_record = StringRecord::from(vec![
            transaction_csv.amount_out.clone(),
            transaction_csv.amount_in.clone(),
            transaction_csv.category.clone(),
            transaction_csv.subcategory.clone(),
            transaction_csv.tag.clone(),
            transaction_csv.date.clone(),
            transaction_csv.note.clone(),
        ]);

        let transaction_csv_deserialized: TransactionCsv =
            transaction_record.deserialize(Some(&header)).unwrap();

        assert_eq!(transaction_csv_deserialized, transaction_csv);
    }
}
