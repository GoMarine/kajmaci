use std::sync::Mutex;
use std::time::Instant;

use lazy_static::lazy_static;
use reqwest::Error;
use scraper::{Html, Selector};

struct Coin {
    name: String,
    price: String,
}

pub fn get_element_text(cell: &scraper::ElementRef) -> String {
    cell.text().collect::<Vec<_>>().join("").trim().to_string()
}

fn make_selector(selector: &str) -> Selector {
    Selector::parse(selector).unwrap()
}

lazy_static! {
    static ref TABLE: Selector = make_selector("table");
    static ref TR: Selector = make_selector("tr");
    static ref TD: Selector = make_selector("td");
}

fn parse_page() -> Result<Coin, Error> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?;
    let res = client
        // .get("https://coinmarketcap.com/all/views/all/")
        .get("https://cryptorank.io/all-coins-list/")
        .send()
        .unwrap()
        .text()
        .unwrap();

    // println!("{}", res2.ok().unwrap().status().is_success());

    let document = Html::parse_document(&res);
    // println!("{}", document.root_element().inner_html());
    let tables = document.select(&TABLE); //.last().unwrap().collect::<ElementRef>();
    for table in tables {
        for row in table.select(&TR) {
            let cells = row.select(&TD).collect::<Vec<_>>();
            if cells.len() > 3 {
                println!(
                    "name:{:?}, price: {:?}",
                    get_element_text(&cells[2]),
                    get_element_text(&cells[3]),
                )
            }
        }
    }

    Ok(Coin {
        name: String::from("BTC"),
        price: String::from("$123"),
    })
}

lazy_static! {
    static ref LAST_REQUEST_MUTEX: Mutex<Option<Instant>> = Mutex::new(None);
    static ref REQUEST_DELAY: std::time::Duration = std::time::Duration::from_millis(500);
}

pub fn do_throttled_request(url: &str) -> Result<String, Error> {
    let mut last_request_mutex = LAST_REQUEST_MUTEX.lock().unwrap();
    let last_request = last_request_mutex.take();
    let now = Instant::now();
    if let Some(last_request) = last_request {
        let duration = now.duration_since(last_request);
        if duration < *REQUEST_DELAY {
            std::thread::sleep(*REQUEST_DELAY - duration);
        }
    }
    let response = reqwest::blocking::get(url)?;
    last_request_mutex.replace(now);
    response.text()
}
fn main() {
    let _res = parse_page().unwrap();
    println!("{}:{}", _res.name, _res.price);
}
