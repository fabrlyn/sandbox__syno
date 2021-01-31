use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate};
use std::collections::HashMap;
use std::env::args;
use tokio::join;

const THESAURUS_URL: &str = "https://www.thesaurus.com/browse";
const YOUR_DICTIONARY_URL: &str = "https://thesaurus.yourdictionary.com";
const MERRIAM_WEBSTER_URL: &str = "https://www.merriam-webster.com/thesaurus";

fn node_to_text(node: Node) -> String {
    node.text().trim().to_lowercase()
}

async fn fetch_from_your_dictionary(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", YOUR_DICTIONARY_URL, word))
        .await
        .find(Class("synonym-link"))
        .map(node_to_text)
        .enumerate()
        .collect::<Vec<_>>()
}

async fn fetch_from_thesaurus(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", THESAURUS_URL, word))
        .await
        .find(Attr("id", "meanings").descendant(Name("li")))
        .map(node_to_text)
        .enumerate()
        .collect::<Vec<_>>()
}

async fn fetch_from_merriam_webster(word: &str) -> Vec<(usize, String)> {
    fetch_document(&format!("{}/{}", MERRIAM_WEBSTER_URL, word))
        .await
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

async fn fetch_document(url: &str) -> Document {
    reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .map(|body| Document::from(body.as_ref()))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let word = args().skip(1).next().unwrap().to_lowercase();

    let (res_1, res_2, res_3) = join!(
        fetch_from_thesaurus(&word),
        fetch_from_merriam_webster(&word),
        fetch_from_your_dictionary(&word)
    );

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
