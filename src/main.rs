use std::f64::consts::PI;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use clap::Parser;
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

#[derive(Parser)]
#[command(name = "IndiaNavi Map Downloader")]
#[command(author = "Bastian Neumann <navi@platinenmacher.tech>")]
#[command(version = "1.0")]
#[command(about = "Loads IndiaNavi map tiles from opentopomap.org or other specified server.", long_about = None)]
struct Cli {
    gpx_path: std::path::PathBuf,
    server_url: Option<String>,
    #[arg(short, long)]
    verbose: bool,
}

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
    let args = Cli::parse();

    let file_path = &args.gpx_path;
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

    let (margin, lon_border, lat_border) = indianavi_gpx_loader::calculate_boundaries(gpx);

    // Provide a custom bar style
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    let mut tasks: Vec<JoinHandle<Result<(), ()>>> = vec![];
    for zoom in [14, 16] {
        let (xrange, yrange) = lonlat2tiles(lon_border, margin, lat_border, zoom);
        for x in xrange {
            for y in yrange.clone() {
                let mut rng = thread_rng();
                let server_addr = args
                    .server_url
                    .as_deref()
                    .unwrap_or(OPENTOPOPMAP_URLS[rng.gen_range(0..OPENTOPOPMAP_URLS.len())]);
                let online_addr = format!("{}/{zoom}/{x}/{y}.png", server_addr);

                // Create a Tokio task for each path
                let pb = pb.clone();
                tasks.push(tokio::spawn(async move {
                    match download_tile(&online_addr).await {
                        Ok(image) => {
                            let file_path_string = format!("MAPS/{zoom}/{x}/{y}.raw");
                            let file_path = Path::new(&file_path_string);
                            let folder_path = file_path.parent().expect("to be a path");
                            fs::create_dir_all(&folder_path).expect("folder can be created");

                            let mut file = BufWriter::new(
                                fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .open(file_path)
                                    .expect("file to be opened for write"),
                            );

                            match file.write_all(&image) {
                                Ok(()) => {
                                    pb.inc(1);
                                    if args.verbose {
                                        pb.println(format!("Load: {online_addr}"));
                                    }
                                }
                                Err(_) => pb.println(format!("Error: {online_addr}")),
                            }
                        }
                        Err(_) => pb.println(format!("Error: {online_addr}")),
                    }
                    Ok(())
                }));
            }
        }
    }
    pb.set_length(tasks.len() as u64);
    futures::future::join_all(tasks).await;
    println!("done. Copy folder MAPS and file track.gpx to the root of your SD card.");
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

