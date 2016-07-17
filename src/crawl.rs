use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;
use super::{Battleplan, load_plan};
use super::errors::*;
use url::Url;

pub fn crawl() -> Result<()> {
    let plan = load_plan()?;
    plan.validate()?;

    let urls = initial_urls_from_plan(&plan);

    let urls_with_distances = urls
        .into_iter()
        .zip(iter::repeat(0))
        .collect::<Vec<_>>();

    let mut urls = VecDeque::from(urls_with_distances);

    let mut facts: HashMap<Url, HashSet<Fact>> = HashMap::new();

    while let Some(url) = urls.pop_front() {
        match learn_about_url(&url, &mut urls, &mut facts) {
            Ok(_) => (),
            Err(e) => {
                add_fact(&mut facts, &url.0, Fact::CrawlError(format!("{}", e)));
            }
        }
        panic!()
    }

    panic!()
}

fn initial_urls_from_plan(plan: &Battleplan) -> Vec<Url> {
    let mut urls = Vec::new();
    for campaign in &plan.campaigns {
        match Url::parse(&campaign.tracking_link) {
            Ok(url) => urls.push(url),
            Err(_) => (/* bogus link */),
        }
    }

    urls
}

type Distance = u32;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
enum Fact {
    CrawlError(String),
    GitHubIssue,
    GitHubPullRequest,
}

fn add_fact(facts: &mut HashMap<Url, HashSet<Fact>>,
            url: &Url,
            fact: Fact) {
    if facts.get(url).is_none() {
        facts.insert(url.clone(), HashSet::new());
    }

    let url_facts = facts.get_mut(url).expect("");
    url_facts.insert(fact);
}

fn learn_about_url(url_d: &(Url, Distance),
                   urls: &mut VecDeque<(Url, Distance)>,
                   facts: &mut HashMap<Url, HashSet<Fact>>) -> Result<()> {
    println!("learning about {}", url_d.0);

    let url = &url_d.0;

    if url.as_str().starts_with("https://github.com") {
    } else {
        panic!()
    }
    
    Ok(())
}

