use std::env;
use std::f64::consts::PI;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use rand::thread_rng;
use rand::Rng;

use indicatif::{ProgressBar, ProgressStyle};

use gpx::read;
use gpx::Gpx;

use reqwest::Client;
use tokio::task::JoinHandle;

use unicode_bom::Bom;

const OPENTOPOPMAP_URLS: [&str; 3] = [
    "https://a.tile.opentopomap.org",
    "https://b.tile.opentopomap.org",
    "https://c.tile.opentopomap.org",
];

async fn download_tile(url: &str) -> Result<Vec<u8>, ()> {
    let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:110.0) Gecko/20100101 Firefox/110.0",
        )
        .build()
        .expect("to act like firefox");
    let resp = client.get(url).send().await.expect("download to success");
    let image =
        indianavi_map_color::convert_image(&resp.bytes().await.expect("download to have bytes"))
            .expect("image to be converted");
    Ok(image)
}

fn lon2tile(lon: f64, zoom: u32) -> u32 {
    let tile = (lon + 180.0) / 360.0 * 2_u32.pow(zoom) as f64;
    tile.floor() as u32
}

fn lat2tile(lat: f64, zoom: u32) -> u32 {
    let tile = (1.0 - lat.to_radians().tan().asinh() / PI) / 2.0 * 2_u32.pow(zoom) as f64;
    tile.floor() as u32
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} track.gpx", &args[0]);
        return;
    }
    let file_path = &args[1];
    let mut file = File::open(&file_path).unwrap();
    let bom = Bom::from(&mut file);

    let file = File::open(&file_path).unwrap();
    let mut reader = BufReader::new(file);
    println!("BOM: {bom}");
    if bom != Bom::Null {
        let mut x = [0; 3];
        let _ = reader.read_exact(&mut x);
        println!("strip 3 bytes from file");
    }

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx: Gpx = read(reader).expect("GPX File can be read");

    let (margin, lon_border, lat_border) = calculate_boundaries(gpx);

    let mut tasks: Vec<JoinHandle<Result<(), ()>>> = vec![];
    for zoom in [14, 16] {
        let (xrange, yrange) = lonlat2tiles(lon_border, margin, lat_border, zoom);
        for x in xrange {
            for y in yrange.clone() {
                let mut rng = thread_rng();
                let online_addr = format!(
                    "{}/{zoom}/{x}/{y}.png",
                    OPENTOPOPMAP_URLS[rng.gen_range(0..OPENTOPOPMAP_URLS.len())]
                );

                // Create a Tokio task for each path
                tasks.push(tokio::spawn(async move {
                    match download_tile(&online_addr).await {
                        Ok(image) => {
                            let file_path_string = format!("tiles/{zoom}/{x}/{y}.raw");
                            let file_path = Path::new(&file_path_string);
                            let folder_path = file_path.parent().expect("to be a path");
                            fs::create_dir_all(&folder_path).expect("folder can be created");

                            let mut file = fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(file_path)
                                .expect("file to be opened for write");

                            match file.write_all(&image) {
                                Ok(()) => {
                                    println!("Load: {online_addr}");
                                }
                                Err(_) => println!("Error: {online_addr}"),
                            }
                        }
                        Err(_) => println!("Error: {online_addr}"),
                    }
                    Ok(())
                }));
            }
        }
    }
    // Provide a custom bar style
    let pb = ProgressBar::new(tasks.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    // TODO: connect the pb with the futures

    futures::future::join_all(tasks).await;
}

fn lonlat2tiles(
    lon_border: [f64; 2],
    margin: u32,
    lat_border: [f64; 2],
    zoom: u32,
) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
    let x_tiles = [
        lon2tile(lon_border[0], zoom) - margin,
        lon2tile(lon_border[1], zoom) + margin,
    ];
    let y_tiles = [
        lat2tile(lat_border[0], zoom) - margin,
        lat2tile(lat_border[1], zoom) + margin,
    ];

    let xrange = x_tiles[0]..x_tiles[1];
    let yrange = y_tiles[0]..y_tiles[1];
    (xrange, yrange)
}

fn calculate_boundaries(gpx: Gpx) -> (u32, [f64; 2], [f64; 2]) {
    // Each GPX file has multiple "tracks", this takes the first one.
    let margin = 5;
    let mut lon_border: [f64; 2] = [90.0, -90.0];
    let mut lat_border: [f64; 2] = [180.0, -180.0];
    for track in &gpx.tracks {
        for s in &track.segments {
            for p in &s.points {
                (lon_border, lat_border) = adjust_boundaries(p, lon_border, lat_border);
            }
        }
    }
    for route in &gpx.routes {
        for p in &route.points {
            (lon_border, lat_border) = adjust_boundaries(p, lon_border, lat_border);
        }
    }

    (margin, lon_border, lat_border)
}

fn adjust_boundaries(
    p: &gpx::Waypoint,
    mut lon_border: [f64; 2],
    mut lat_border: [f64; 2],
) -> ([f64; 2], [f64; 2]) {
    let x = p.point().x();
    let y = p.point().y();
    if x < lon_border[0] {
        lon_border[0] = x;
    } else if x > lon_border[1] {
        lon_border[1] = x;
    }
    if y < lat_border[0] {
        lat_border[0] = y;
    } else if y > lat_border[1] {
        lat_border[1] = y;
    }
    (lon_border, lat_border)
}
