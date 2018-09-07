extern crate select;
extern crate chrono;

use self::select::document::Document;
use self::select::node::Node;
use self::select::predicate::Element;

use self::chrono::{DateTime, Utc};

use bson::Document as BsonDocument;

pub struct DavisDataDownloader {
    id: String,
}

impl DavisDataDownloader {
    pub fn with_id(id: &str) -> DavisDataDownloader {
        DavisDataDownloader { id: id.to_owned() }
    }

    pub fn download(self) -> Result<DavisData, ::reqwest::Error> {
        let html = ::reqwest::get(&format!(
            "http://web.directemar.cl/met/jturno/estaciones/{}/index.htm",
            self.id
        ))?
            .text()?;
        Ok(DavisData::from_document(
            self.id.clone(),
            DavisDataDocument(self.id, Document::from(html.as_ref())),
        ))
    }
}

pub struct DavisData {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub temp: f64,
    pub windchill: f64,
    pub heat_index: f64,
    pub humidity: f64,
    pub dew_point: f64,
    pub rainfall_lasthour: f64,
    pub rainfall_thismonth: f64,
    pub rainfall_rate: f64,
    pub rainfall_thisyear: f64,
    pub rainfall_last: DateTime<Utc>,
    pub rainfall_today: f64,
    pub has_wind_data: bool,
    pub wind_direction: f64,
    pub wind_speed_gust: f64,
    pub wind_speed_avg: f64,
    pub barometer: f64,
}

impl DavisData {
    fn from_document(id: String, doc: DavisDataDocument) -> DavisData {
        DavisData {
            id,
            ts: Utc::now(),
            temp: doc.temp(),
            windchill: doc.windchill(),
            heat_index: doc.heat_index(),
            humidity: doc.humidity(),
            dew_point: doc.dew_point(),
            rainfall_lasthour: doc.rainfall_lasthour(),
            rainfall_thismonth: doc.rainfall_thismonth(),
            rainfall_rate: doc.rainfall_rate(),
            rainfall_thisyear: doc.rainfall_thisyear(),
            rainfall_last: doc.rainfall_last(),
            rainfall_today: doc.rainfall_today(),
            has_wind_data: doc.has_wind_data(),
            wind_direction: doc.wind_direction(),
            wind_speed_gust: doc.wind_speed_gust(),
            wind_speed_avg: doc.wind_speed_avg(),
            barometer: doc.barometer(),
        }
    }
}

impl<'a> From<&'a DavisData> for BsonDocument {
    fn from(davis: &DavisData) -> BsonDocument {
        doc! {
            "_id": ::bson::oid::ObjectId::new().unwrap(),
            "ts": davis.ts,
            "station": davis.id.clone(),
            "temp": davis.temp,
            "windchill": davis.windchill,
            "heat_index": davis.heat_index,
            "humidity": davis.humidity,
            "dew_point": davis.dew_point,
            "rainfall_lasthour": davis.rainfall_lasthour,
            "rainfall_thismonth": davis.rainfall_thismonth,
            "rainfall_rate": davis.rainfall_rate,
            "rainfall_thisyear": davis.rainfall_thisyear,
            "rainfall_last": davis.rainfall_last,
            "rainfall_today": davis.rainfall_today,
            "has_wind_data": davis.has_wind_data,
            "wind_direction": davis.wind_direction,
            "wind_speed_gust": davis.wind_speed_gust,
            "wind_speed_avg": davis.wind_speed_avg,
            "barometer": davis.barometer,
        }
    }
}

struct DavisDataDocument(String, Document);

impl DavisDataDocument {
    fn table_element(&self, name: &str, replace: &str) -> String {
        let gpar = self.1 // doc
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

    fn temp(&self) -> f64 {
        let elem = self.table_element("Temperature", "\u{a0}°C").replace(
            ",",
            ".",
        );
        elem.parse().expect(&format!(
            "[{}] Could not parse temp: {}",
            &self.0,
            elem
        ))
    }

    fn windchill(&self) -> f64 {
        let elem = self.table_element("Windchill", "\u{a0}°C").replace(
            ",",
            ".",
        );
        elem.parse().expect(&format!(
            "[{}] Could not parse windchill: {}",
            &self.0,
            elem
        ))
    }

    fn heat_index(&self) -> f64 {
        let elem = self.table_element("Heat Index", "\u{a0}°C").replace(
            ",",
            ".",
        );
        elem.parse().expect(&format!(
            "[{}] Could not parse heat_index: {}",
            &self.0,
            elem
        ))
    }

    fn humidity(&self) -> f64 {
        let elem = self.table_element("Humidity", "%").replace(",", ".");
        elem.parse::<f64>().expect(&format!(
            "[{}] Could not parse 'humidity': {}",
            &self.0,
            elem
        )) / 100.0
    }

    fn dew_point(&self) -> f64 {
        let elem = self.table_element("Dew\u{a0}Point ", "\u{a0}°C").replace(
            ",",
            ".",
        );
        elem.parse().expect(&format!(
            "[{}] Could not parse 'dew_point': {}",
            &self.0,
            elem
        ))
    }

    fn rainfall_lasthour(&self) -> f64 {
        let elem = self.table_element("Rainfall\u{a0}Last Hour", "\u{a0}mm")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'rainfall_lasthour': {}",
            &self.0,
            elem
        ))
    }

    fn rainfall_thismonth(&self) -> f64 {
        let elem = self.table_element("Rainfall\u{a0}This\u{a0}Month", "\u{a0}mm")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'rainfall_thismonth': {}",
            &self.0,
            elem
        ))
    }

    fn rainfall_rate(&self) -> f64 {
        let elem = self.table_element("Rainfall\u{a0}Rate", "\u{a0}mm/hr")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'rainfall_rate': {}",
            &self.0,
            elem
        ))
    }

    fn rainfall_thisyear(&self) -> f64 {
        let elem = self.table_element("Rainfall\u{a0}This\u{a0}Year", "\u{a0}mm")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'rainfall_thisyear': {}",
            &self.0,
            elem
        ))
    }

    fn rainfall_last(&self) -> DateTime<Utc> {
        let dt_with_tz = format!(
            "{} -03:00", // Fix in case code is running in foreign server
            self.table_element("Last rainfall", "\u{a0}mm").as_str()
        );
        DateTime::parse_from_str(&dt_with_tz, "%Y-%m-%d %H:%M %:z")
            .expect(&format!(
                "[{}] Could not parse 'rainfall_last': {}",
                &self.0,
                dt_with_tz
            ))
            .with_timezone(&Utc)
    }

    fn rainfall_today(&self) -> f64 {
        let elem = self.table_element("Rainfall\u{a0}Today", "\u{a0}mm")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'rainfall_today': {}",
            &self.0,
            elem
        ))
    }

    fn has_wind_data(&self) -> bool {
        !self.table_element("Wind Bearing", "").ends_with("---")
    }

    fn wind_direction(&self) -> f64 {
        let wind = self.table_element("Wind Bearing", "").replace(",", ".");
        wind[..wind.find("°").expect(&format!(
            "[{}] Could not find '°' index",
            &self.0
        ))].parse()
            .expect(&format!(
                "[{}] Could not parse 'wind_direction': {}",
                &self.0,
                wind
            ))
    }

    fn wind_speed_gust(&self) -> f64 {
        let elem = self.table_element("Wind\u{a0}Speed\u{a0}(gust)", "\u{a0}kts")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'wind_speed_gust': {}",
            &self.0,
            elem
        ))
    }

    fn wind_speed_avg(&self) -> f64 {
        let elem = self.table_element("Wind\u{a0}Speed\u{a0}(avg)", "\u{a0}kts")
            .replace(",", ".");
        elem.parse().expect(&format!(
            "[{}] Could not parse 'wind_speed_avg': {}",
            &self.0,
            elem
        ))
    }

    fn barometer(&self) -> f64 {
        let elem = self.table_element("Barometer\u{a0}", "\u{a0}hPa").replace(
            ",",
            ".",
        );
        elem.parse().expect(&format!(
            "[{}] Could not parse 'barometer': {}",
            &self.0,
            elem
        ))
    }
}