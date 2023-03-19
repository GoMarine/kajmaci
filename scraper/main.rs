use lazy_static::lazy_static;
use reqwest::Error;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Serialize, Deserialize)]
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
    static ref TABLE: Selector = make_selector("table.sort");
    static ref TR: Selector = make_selector("tr");
    static ref TD: Selector = make_selector("td");
}

fn parse_page() -> Result<Vec<Coin>, Error> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?;

    let mut coins: Vec<Coin> = Vec::new();

    for i in 1..2 {
        let res = client
            .get(format!("https://www.coingecko.com/?page={}", i))
            .send()
            .unwrap()
            .text()
            .unwrap();

        let document = Html::parse_document(&res);
        let tables = document.select(&TABLE);
        for table in tables {
            for row in table.select(&TR) {
                let cells = row.select(&TD).collect::<Vec<_>>();
                if cells.len() > 3 {
                    let coin = Coin {
                        name: get_element_text(&cells[2]).trim().replace("\n", ""),
                        price: get_element_text(&cells[3]),
                    };
                    coins.push(coin);
                }
            }
        }
    }

    Ok(coins)
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
    match parse_page() {
        Ok(coins) => {
            let json_str = serde_json::to_string(&coins).unwrap();
            let mut file = File::create("coins.json").unwrap();
            file.write_all(json_str.as_bytes()).unwrap();
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
