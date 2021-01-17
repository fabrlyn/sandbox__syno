use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate};
use std::collections::HashMap;
use std::env::args;

const THESAURUS_URL: &str = "https://www.thesaurus.com/browse";
const YOUR_DICTIONARY_URL: &str = "https://thesaurus.yourdictionary.com";
const MERRIAM_WEBSTER_URL: &str = "https://www.merriam-webster.com/thesaurus";

fn node_to_text(node: Node) -> String {
    node.text().trim().to_lowercase()
}

fn fetch_from_your_dictionary(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", YOUR_DICTIONARY_URL, word))
        .find(Class("synonym-link"))
        .map(node_to_text)
        .enumerate()
        .collect::<Vec<_>>()
}

fn fetch_from_thesaurus(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", THESAURUS_URL, word))
        .find(Attr("id", "meanings").descendant(Name("li")))
        .map(node_to_text)
        .enumerate()
        .collect::<Vec<_>>()
}

fn fetch_from_merrian_webster(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", MERRIAM_WEBSTER_URL, word))
        .find(
            Class("syn-list")
                .descendant(Class("mw-list"))
                .child(Name("li"))
                .child(Name("a")),
        )
        .map(node_to_text)
        .enumerate()
        .collect::<Vec<_>>()
}

fn fetch_document(url: &str) -> Document {
    reqwest::blocking::get(url)
        .unwrap()
        .text()
        .map(|body| Document::from(body.as_ref()))
        .unwrap()
}

fn main() {
    let word = args().skip(1).next().unwrap().to_lowercase();

    let res_1 = fetch_from_thesaurus(&word);
    let res_2 = fetch_from_merrian_webster(&word);
    let res_3 = fetch_from_your_dictionary(&word);

    let mut result = res_1
        .into_iter()
        .chain(res_2.into_iter())
        .chain(res_3.into_iter())
        .fold(HashMap::new(), |mut acc, (ranking, synonym)| {
            let entry = acc.entry(synonym).or_insert(0);
            *entry += ranking;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    result.sort_by(|(_, a), (_, b)| a.cmp(b));

    println!("Synonyms for {}", word);
    result
        .iter()
        .take(10)
        .for_each(|(synonym, _)| println!("- {}", synonym));
}
