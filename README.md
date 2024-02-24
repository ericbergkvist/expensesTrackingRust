# expensesTrackingRust

Manage and analyse CSV expenses using a Rust codebase.

## Usage

At the moment, run `cargo run` in the `expenses_tracking` folder.

## Todos (and vision)

- Function to write transactions to a CSV file
- Function to rename categories and sub-categories
- Each category and sub-category needs a date associated with it, such that data analysis can in the future account for it.
  - Edge-case: how should we deal with a (sub-)category that was created, then deleted at some point, and then recreated? We would then need more than a start and end date. Should it be a vector of dates?
  - It might make sense to make categories and sub-categories structs (attributes: name, date of creation, date of deletion?)

## Known flaws

- General errors (`Box<dyn Error>`) are used everywhere and should be replaced when it makes sense
