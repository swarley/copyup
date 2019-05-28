extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate strsim;

use serde::Deserialize;
use strsim::damerau_levenshtein;

const LICENSE_LIST_URL: &str = "https://spdx.org/licenses/licenses.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseEntry {
    pub reference: String,
    pub is_deprecated_license_id: bool,
    pub details_url: String,
    pub reference_number: String,
    pub name: String,
    pub license_id: String,
    pub see_also: Vec<String>,
    pub is_osi_approved: bool,
}

pub enum Match<T> {
    Exact(T),
    Closest(T),
    None,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseDetails {
    pub is_deprecated_license_id: bool,
    pub is_fsf_libre: Option<bool>,
    pub license_text: String,
    pub standard_license_template: String,
    pub name: String,
    pub license_id: String,
    pub see_also: Vec<String>,
    pub is_osi_approved: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseList {
    pub license_list_version: String,
    pub licenses: Vec<LicenseEntry>,
}

pub fn fetch_list() -> Result<LicenseList, Box<std::error::Error>> {
    let resp: LicenseList = reqwest::get(LICENSE_LIST_URL)?.json()?;
    Ok(resp)
}

impl LicenseList {
    pub fn find_match(&self, match_str: &str) -> Match<&LicenseEntry> {
        let match_str = match_str.to_lowercase();

        let scores: Vec<(i32, &String)> = self
            .licenses
            .iter()
            .map(|entry| {
                (
                    damerau_levenshtein(&match_str.to_lowercase(), &entry.license_id.to_lowercase())
                        as i32,
                    &entry.license_id,
                )
            })
            .collect();

        match scores.iter().min() {
            Some((score, id)) if *score == 0 => {
                return Match::Exact(
                    self.licenses
                        .iter()
                        .find(|entry| entry.license_id == **id)
                        .unwrap(),
                );
            }
            Some((_, id)) => {
                return Match::Closest(
                    self.licenses
                        .iter()
                        .find(|entry| entry.license_id == **id)
                        .unwrap(),
                );
            }
            None => Match::None,
        }
    }

    pub fn display_list(&self) {
        let max_size = self
            .licenses
            .iter()
            .map(|entry| entry.license_id.chars().count())
            .max()
            .unwrap();
        for entry in &self.licenses {
            println!(
                "{:width$} -- {}",
                entry.license_id,
                entry.name,
                width = max_size
            )
        }
    }
}

impl LicenseEntry {
    pub fn fetch_text(&self) -> Result<String, Box<std::error::Error>> {
        let resp: LicenseDetails = reqwest::get(&self.details_url)?.json()?;
        Ok(resp.license_text)
    }
}
