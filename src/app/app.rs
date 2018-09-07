use crawler::{DavisDataDownloader, DavisData};
use persist::MongoConnection;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;
use logger::Logger;

pub struct AppSettings {
    pub poll_interval: i32,
    pub targets: Vec<String>,
    pub connection_string: String,
    pub database: String,
    pub collection: String,
}

pub struct App {
    settings: AppSettings,
}

impl App {
    pub fn new(settings: AppSettings) -> App {
        App { settings }
    }

    /*
     * Main thread schedules downloads
     *  - each download on its own thread
     *  - separate thread to persist downloads
     */
    pub fn start(self) {
        let (tx, rx): (Sender<DavisData>, Receiver<DavisData>) = channel();

        // Thread params
        let connstr = self.settings.connection_string.clone();
        let db = self.settings.database.clone();
        let coll = self.settings.collection.clone();
        thread::spawn(move || {
            let logger = Logger::with_tag("MongoConnection");
            loop {
                logger.info("Connecting with mongo instance");
                match MongoConnection::with_connection_string(&connstr) {
                    Ok(conn) => {
                        loop {
                            match rx.recv() {
                                Ok(davis_data) => {
                                    logger.info(&format!("Persisting data for {}", davis_data.id));
                                    if let Err(err) = conn.insert(&db, &coll, davis_data) {
                                        logger.error(&format!("{}", err));
                                        break;
                                    }
                                }
                                Err(err) => {
                                    logger.error(&format!("{:?}", err));
                                    continue;
                                }
                            };
                        }
                    }
                    Err(err) => logger.error(&format!("{}", err)),
                }
            }
        });

        loop {
            for target in &self.settings.targets {
                let tx = tx.clone();
                let target_arg = target.clone();
                thread::spawn(move || {
                    let logger = Logger::with_tag("Downloader");
                    logger.info(&format!("Downloading latest data for {}", &target_arg));
                    match DavisDataDownloader::with_id(&target_arg).download() {
                        Ok(davis_data) => {
                            if let Err(err) = tx.send(davis_data) {
                                logger.error(&format!("{:?}", err));
                            }
                        }
                        Err(e) => logger.error(&format!("{:?}", e)),
                    }
                });
            }
            thread::sleep(Duration::from_secs(self.settings.poll_interval as u64));
        }
    }
}