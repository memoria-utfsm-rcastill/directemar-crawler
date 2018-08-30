extern crate reqwest;
extern crate select;
extern crate chrono;

use select::document::Document;
use select::node::Node;
use select::predicate::Element;

use chrono::DateTime;

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

    fn download(self) -> Result<DavisData, reqwest::Error> {
        let html = reqwest::get(&format!(
            "http://web.directemar.cl/met/jturno/estaciones/{}/index.htm",
            self.id
        ))?
            .text()?;
        Ok(DavisData { doc: Document::from(html.as_ref()) })
    }
}

struct DavisData {
    doc: Document,
}

impl DavisData {
    fn table_element(&self, name: &str, replace: &str) -> String {
        let gpar = self.doc
            // Get node with html() temperature
            .find(|node: &Node| node.text() == name)
            .next()
            .unwrap()
            // Get table element <tr>
            .parent()
            .unwrap();

        let mut expecting_value = false;
        for uncle in gpar.children() {
            if uncle.is(Element) {
                if expecting_value {
                    return uncle.text().replace(replace, "");
                } else if uncle.text() == name {
                    expecting_value = true;
                }
            }
        }

        unreachable!(format!(
            "Fell off the for loop @ table_element({:?}, {:?})",
            name,
            replace
        ));
    }

    pub fn temp(&self) -> f64 {
        self.table_element("Temperature", "\u{a0}°C")
            .parse()
            .unwrap()
    }

    pub fn windchill(&self) -> f64 {
        self.table_element("Windchill", "\u{a0}°C")
            .parse()
            .unwrap()
    }

    pub fn heat_index(&self) -> f64 {
        self.table_element("Heat Index", "\u{a0}°C")
            .parse()
            .unwrap()
    }

    pub fn humidity(&self) -> f64 {
        self.table_element("Humidity", "%").parse::<f64>().unwrap() / 100.0
    }

    pub fn dew_point(&self) -> f64 {
        self.table_element("Dew\u{a0}Point ", "\u{a0}°C")
            .parse()
            .unwrap()
    }

    pub fn rainfall_lasthour(&self) -> f64 {
        self.table_element("Rainfall\u{a0}Last Hour", "\u{a0}mm")
            .parse()
            .unwrap()
    }

    pub fn rainfall_thismonth(&self) -> f64 {
        self.table_element("Rainfall\u{a0}This\u{a0}Month", "\u{a0}mm")
            .parse()
            .unwrap()
    }

    pub fn rainfall_rate(&self) -> f64 {
        self.table_element("Rainfall\u{a0}Rate", "\u{a0}mm/hr")
            .parse()
            .unwrap()
    }

    pub fn rainfall_thisyear(&self) -> f64 {
        self.table_element("Rainfall\u{a0}This\u{a0}Year", "\u{a0}mm")
            .parse()
            .unwrap()
    }

    pub fn rainfall_last(&self) -> DateTime<chrono::FixedOffset> {
        // Set Chile timezone
        let date_with_timezone = format!(
            "{} -0300",
            self.table_element("Last rainfall", "\u{a0}mm").as_str()
        );
        DateTime::parse_from_str(date_with_timezone.as_str(), "%Y-%m-%d %H:%M %z").unwrap()
    }

    pub fn rainfall_today(&self) -> f64 {
        self.table_element("Rainfall\u{a0}Today", "\u{a0}mm")
            .parse()
            .unwrap()
    }

    pub fn has_wind_data(&self) -> bool {
        !self.table_element("Wind Bearing", "").ends_with("---")
    }

    pub fn wind_direction(&self) -> f64 {
        let wind = self.table_element("Wind Bearing", "");
        wind[..wind.find("°").unwrap()].parse().unwrap()
    }

    pub fn wind_speed_gust(&self) -> f64 {
        self.table_element("Wind\u{a0}Speed\u{a0}(gust)", "\u{a0}kts")
            .parse()
            .unwrap()
    }

    pub fn wind_speed_avg(&self) -> f64 {
        self.table_element("Wind\u{a0}Speed\u{a0}(avg)", "\u{a0}kts")
            .parse()
            .unwrap()
    }

    pub fn barometer(&self) -> f64 {
        self.table_element("Barometer\u{a0}", "\u{a0}hPa")
            .parse()
            .unwrap()
    }
}

fn main() {
    let davis_data = match DavisDataDownloader::with_id("valparaiso").download() {
        Ok(dd) => dd,
        Err(e) => {
            handle_reqwest_err(e);
            return;
        }
    };
    println!("temperature    = {}", davis_data.temp());
    println!("windchill      = {}", davis_data.windchill());
    println!("heat index     = {}", davis_data.heat_index());
    println!("humidity       = {}", davis_data.humidity());
    println!("dew point      = {}", davis_data.dew_point());
    println!("rainfall lh    = {}", davis_data.rainfall_lasthour());
    println!("rainfall tm    = {}", davis_data.rainfall_thismonth());
    println!("rainfall ra    = {}", davis_data.rainfall_rate());
    println!("rainfall ty    = {}", davis_data.rainfall_thisyear());
    println!("rainfall la    = {:?}", davis_data.rainfall_last());
    println!("rainfall to    = {}", davis_data.rainfall_today());
    if davis_data.has_wind_data() {
        println!("wind direction = {}", davis_data.wind_direction());
        println!("wind speed (g) = {}", davis_data.wind_speed_gust());
        println!("wind speed (a) = {}", davis_data.wind_speed_avg());
    }
    println!("barometer      = {}", davis_data.barometer());
}