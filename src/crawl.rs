
use serde_yaml;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;
use super::{Battleplan, load_plan};
use super::errors::*;
use url::Url;
use gh::models::IssueFromJson;
use DATA_DIR;
use std::path::PathBuf;
use std::io::Write;
use regex::Regex;
use std::fs::{self, File};

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Fact {
    CrawlError(String),
    GitHubIssue(IssueFromJson),
    GitHubPullRequest,
}

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
        if url.1 > MAX_DISTANCE { continue }
        match learn_about_url(&url, &mut urls, &mut facts) {
            Ok(_) => (),
            Err(e) => {
                add_fact(&mut facts, &url.0, Fact::CrawlError(format!("{}", e)));
            }
        }
    }

    let facts_s = serde_yaml::to_string(&facts).chain_err(|| "encoding url facts")?;

    let data_dir = &PathBuf::from(DATA_DIR).join("gen");
    fs::create_dir_all(data_dir)?;
    let crawl_file = &data_dir.join("crawl.yml");
    let mut f = File::create(crawl_file)?;
    writeln!(f, "{}", facts_s)?;

    println!("{} updated", crawl_file.display());

    Ok(())
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

const MAX_DISTANCE: u32 = 5;

type Distance = u32;

fn add_fact(facts: &mut HashMap<Url, HashSet<Fact>>,
            url: &Url,
            fact: Fact) {
    if facts.get(&url).is_none() {
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
        let (new_urls, new_facts) = learn_about_github_url(url)?;

        for new_url in new_urls {
            urls.push_back((new_url, url_d.1 + 1));
        }

        for (new_url, new_fact) in new_facts {
            add_fact(facts, &new_url, new_fact);
        }
    } else {
        verr!("URL not understood: {}", url);
    }

    Ok(())
}


fn learn_about_github_url(url: &Url) -> Result<(Vec<Url>, Vec<(Url, Fact)>)> {
    if url.as_str().contains("/issues/") {
        learn_about_github_issue(url)
    } else {
        verr!("GitHub URL not understood: {}", url);
        Ok((Vec::new(), Vec::new()))
    }
}

fn learn_about_github_issue(url: &Url) -> Result<(Vec<Url>, Vec<(Url, Fact)>)> {
        
    use gh::client::Client;

    let mut new_urls = Vec::new();
    let mut new_facts = Vec::new();

    let (org, repo, number) = parse_gh_issue(url)?;

    println!("{} / {} / {}", org, repo, number);

    let client = Client::new();

    let issue = client.fetch_issue(&org, &repo, &number)?;

    if let Some(ref body) = issue.body {
        // Match "rust-lang/rfcs#more-than-one-digit"
        let rfc_ref_re = Regex::new(r"rust-lang/rfcs#(\d{1,})").expect("");

        for line in body.lines() {
            if rfc_ref_re.is_match(line) {
                println!("WOO {}", line);
                for cap in rfc_ref_re.captures_iter(line) {
                    println!("WOOWHOO {}", cap.at(1).expect(""));
                }
            }
        }
    }

    new_facts.push((url.clone(), Fact::GitHubIssue(issue)));

    Ok((new_urls, new_facts))
}

fn parse_gh_issue(url: &Url) -> Result<(String, String, String)> {
    // Parse org/repo/# from the URL. FIXME: regex
    let site = "https://github.com/";
    assert!(url.as_str().starts_with(site));
    let after_site = &url.as_str()[site.len()..];
    assert!(after_site.contains("/"));
    assert!(!after_site.starts_with("/"));
    let next_slash = after_site.find('/').unwrap();
    let owner = &after_site[..next_slash];
    let after_owner = &after_site[next_slash + 1..];
    assert!(after_owner.contains("/"));
    assert!(!after_owner.starts_with("/"));
    let next_slash = after_owner.find('/').unwrap();
    let repo = &after_owner[..next_slash];
    let after_repo = &after_owner[next_slash + 1..];
    assert!(after_repo.starts_with("issues/"));
    let after_issue = &after_repo["issues/".len()..];
    let number;
    if let Some(i) = after_issue.find('#') {
        number = &after_issue[..i];
    } else {
        number = after_issue;
    }

    Ok((owner.into(), repo.into(), number.into()))
}
