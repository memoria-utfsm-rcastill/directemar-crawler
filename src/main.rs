extern crate reqwest;
extern crate select;

use select::document::Document;
use select::node::Node;

fn handle_reqwest_err(e: reqwest::Error) {
    if e.is_http() {
        match e.url() {
            None => println!("No Url given"),
            Some(url) => println!("Problem making request to: {}", url),
        }
    }
    // Inspect the internal error and output it
    if e.is_serialization() {
        let serde_error = match e.get_ref() {
            None => return,
            Some(err) => err,
        };
        println!("problem parsing information {}", serde_error);
    }
    if e.is_redirect() {
        println!("server redirecting too many times or making loop");
    }
}

struct DavisDataDownloader {
    id: String,
}

impl DavisDataDownloader {
    fn with_id(id: &str) -> DavisDataDownloader {
        DavisDataDownloader { id: id.to_owned() }
    }

    fn download(self) -> Result<DavisData, ()> {
        match reqwest::get(&format!(
            "http://web.directemar.cl/met/jturno/estaciones/{}/index.htm",
            self.id
        )) {
            Ok(mut r) => {
                match r.text() {
                    Ok(html) => Ok(DavisData { doc: Document::from(html.as_ref()) }),
                    Err(e) => Err(handle_reqwest_err(e)),
                }
            }
            Err(e) => Err(handle_reqwest_err(e)),
        }
    }
}

struct DavisData {
    doc: Document,
}

impl DavisData {
    fn get_temperature(&self) -> f64 {
        self.doc
            // Get node with html() temperature
            .find(|node: &Node| node.html() == "Temperature")
            .next()
            .unwrap()
            // Get wrapper element <td>
            .parent()
            .unwrap()
            // Get table <tr>
            .parent()
            .unwrap()
            // Get uncle
            .children()
            .nth(3)
            .unwrap()
            // Get sibling (temp <td> element)
            .children()
            .next()
            .unwrap()
            .text()
            // Remove non-digit
            .replace("\u{a0}°C", "")
            .parse()
            .unwrap()
    }
}

fn main() {
    let temp = match DavisDataDownloader::with_id("valparaiso").download() {
        Ok(davis_data) => davis_data.get_temperature(),
        Err(_) => {
            println!("Could not download data");
            return;
        }
    };
    println!("temp = {:?}", temp);
}