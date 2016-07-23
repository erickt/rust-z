use {Battleplan, load_plan};
use chrono::{UTC};
use errors::*;
use crawl::{UrlFacts, load_url_facts, FactSetExt};
use std::collections::HashMap;
use url::Url;
use regex::Regex;
use std::ops::Deref;
use std::convert::TryFrom;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash)]
struct Campaign {
    rfc: Option<RfcInfo>,
    fcp: Option<Url>,
    completed: bool,
    last_updated: Option<(String, u32)>, // (Y-m-d, days-since-update)
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash)]
struct RfcInfo {
    num: u32,
    pr: Url,
}

pub fn ponder() -> Result<()> {
    let plan = load_plan()?;
    plan.validate()?;

    let ref url_facts = load_url_facts()?;

    let campaign_urls = campaign_urls_from_plan(&plan);

    let mut campaigns = HashMap::new();
    
    for (ref campaign_id, ref url) in campaign_urls {
        info!("calculating campaign for {}", campaign_id);

        if url_facts.get(url).is_none() {
            warn!("no crawl info for {}", url);
            continue;
        }

        let rfc_info = get_rfc_info(url_facts, url);
        let last_updated = get_last_updated(url_facts, url);

        let campaign = Campaign {
            rfc: rfc_info,
            fcp: None,
            completed: false,
            last_updated: last_updated,
        };

        campaigns.insert(campaign_id.to_string(), campaign);
    }

    super::write_yaml("campaigns", campaigns)?;

    Ok(())
}

fn campaign_urls_from_plan(plan: &Battleplan) -> Vec<(String, Url)> {
    let mut cs = Vec::new();
    for campaign in &plan.campaigns {
        match Url::parse(&campaign.tracking_link) {
            Ok(url) => cs.push((campaign.id.clone(), url)),
            Err(_) => (/* bogus link */),
        }
    }

    cs
}

fn get_rfc_info(url_facts: &UrlFacts, campaign_url: &Url) -> Option<RfcInfo> {
    if url_facts.get(campaign_url).is_none() {
        return None;
    }

    let facts = &url_facts[campaign_url];
    let rfc_number;

    if let Some(ref issue) = facts.gh_issue() {
        let issue_body = issue.body.as_ref().map(Deref::deref).unwrap_or("");
        let rfc_numbers = parse_rfc_numbers(&issue_body);

        if rfc_numbers.len() == 0 {
            return None;
        }

        if rfc_numbers.len() > 1 {
            warn!("multiple RFC candidates for {}: {:?}", campaign_url, rfc_numbers);
        }

        rfc_number = rfc_numbers[0];
    } else {
        return None;
    }

    let rfc_url = Url::parse(&format!("https://github.com/rust-lang/rfcs/pulls/{}", rfc_number)).expect("");

    Some(RfcInfo {
        num: rfc_number,
        pr: rfc_url
    })
}

fn get_last_updated(url_facts: &UrlFacts, campaign_url: &Url) -> Option<(String, u32)> {
    if url_facts.get(campaign_url).is_none() {
        return None;
    }

    let facts = &url_facts[campaign_url];

    // TODO: This should probably also consider sub-tasks, and certain other
    // related URLs as part of the last updated time

    if let Some(ref issue) = facts.gh_issue() {
        let date = format!("{}", issue.updated_at.format("%Y-%m-%d"));
        let days_since_update = (UTC::now() - issue.updated_at).num_days();
        let days_since_update = u32::try_from(days_since_update).unwrap_or(0);
        Some((date, days_since_update))
    } else {
        None
    }
}
fn parse_rfc_numbers(text: &str) -> Vec<u32> {
    // Match "rust-lang/rfcs#more-than-one-digit"
    let rfc_ref_re = Regex::new(r"rust-lang/rfcs#(\d+)").expect("");
    let rfc_url_re = Regex::new(r"https://github.com/rust-lang/rfcs/pull/(\d+)").expect("");
    let mut rfc_numbers = vec!();
    
    for line in text.lines() {
        if let Some(cap) = rfc_ref_re.captures(line) {
            let rfc_num_str = cap.at(1).expect("");
            if let Ok(n) = str::parse(rfc_num_str) {
                rfc_numbers.push(n);
            } else {
                warn!("weird rfc number didn't parse {}", rfc_num_str);
            }
        } else if let Some(cap) = rfc_url_re.captures(line) {
            let rfc_num_str = cap.at(1).expect("");
            if let Ok(n) = str::parse(rfc_num_str) {
                rfc_numbers.push(n);
            } else {
                warn!("weird rfc number didn't parse {}", rfc_num_str);
            }
        }
    }

    rfc_numbers
}
