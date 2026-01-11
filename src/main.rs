use std::{collections::HashMap, error::Error, fmt::Display};

use csv::{ReaderBuilder, StringRecord};

const FILE: &'static str = "data.csv";
const CATEGORY: &'static str = "health";

struct Entry {
    year: String,
    category: String,
    cost: f32,
    note: Option<String>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(note) = &self.note {
            write!(
                f,
                "[{}] {}: ${:.2} ({})",
                &self.year, self.category, self.cost, note
            )
        } else {
            write!(f, "[{}] {}: ${:.2}", &self.year, self.category, self.cost)
        }
    }
}

impl Entry {
    pub fn new(record: &StringRecord) -> Option<Entry> {
        if record
            .get(1)
            .map(|category| !category.eq_ignore_ascii_case(CATEGORY))
            .unwrap_or(false)
        {
            return None;
        }

        let Some(category) = record.get(2).map(|str| {
            let category = str.trim().to_owned();
            if category.is_empty() {
                "Unknown".to_string()
            } else {
                category
            }
        }) else {
            return None;
        };

        let Some(Ok(cost)) = record.get(4).map(|str| str.parse::<f32>()) else {
            return None;
        };

        let note = record
            .get(10)
            .filter(|str| !str.is_empty())
            .map(|str| str.trim().to_owned());

        let date = record.get(7).map(|str| {
            str.split("-")
                .map(|str| str.to_owned())
                .collect::<Vec<String>>()
        });

        let Some(Some(year)) = date.as_ref().map(|date| date.first()) else {
            return None;
        };

        Some(Entry {
            year: year.to_string(),
            category,
            cost,
            note,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let reader = ReaderBuilder::new().from_path(FILE)?;
    let mut entries: Vec<Entry> = reader
        .into_records()
        .filter_map(|record| record.ok())
        .filter_map(|record| Entry::new(&record))
        .collect();

    entries.sort_by(|entry1, entry2| {
        entry1
            .year
            .cmp(&entry2.year)
            .then(entry1.category.cmp(&entry2.category))
            .then(entry1.cost.total_cmp(&entry2.cost))
    });

    let mut cat_totals: HashMap<String, HashMap<String, f32>> = HashMap::new();
    entries.iter().for_each(|entry| {
        if let Some(year) = cat_totals.get_mut(&entry.year) {
            if let Some(key_val) = year.get_mut(&entry.category) {
                *key_val += entry.cost;
            } else {
                year.insert(entry.category.clone(), entry.cost);
            }
        } else {
            let mut fresh = HashMap::new();
            fresh.insert(entry.category.clone(), entry.cost);
            cat_totals.insert(entry.year.clone(), fresh);
        }
    });

    let year_totals: HashMap<String, f32> = cat_totals
        .iter()
        .map(|year| {
            (
                year.0.to_owned(),
                year.1.iter().map(|(_, cost)| cost).sum::<f32>(),
            )
        })
        .collect();

    entries.into_iter().for_each(|record| {
        println!("{}", record);
    });

    println!("\nCategory Totals:");

    cat_totals.iter().for_each(|year| {
        println!("\n{}\n", year.0);
        year.1.iter().for_each(|entry| {
            println!("{}: ${}", entry.0, entry.1);
        });
    });

    println!("\nYear Totals:\n");

    year_totals.iter().for_each(|total| {
        println!("{}: ${}", total.0, total.1);
    });

    Ok(())
}
