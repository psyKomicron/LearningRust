pub mod console;
pub mod threaded_queue;
pub mod tests;
pub mod prime;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::prelude::Distribution;
use sqlite;
use sqlite::State;
use reqwest;
use tokio::time::Instant;
use std::env;
use std::ops::Range;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::Write};
use std::path::Path;
use url::Url;
use rand;

use crate::threaded_queue::Processor;

struct Ticket
{
    /*
     - id               0
     - date_created     1
     - download_uri     2
     - ticket_token     3
     - file_size        4
     - requires_auth    5
     - is_done          6
     - file_name        7
     - download_date    8
     */
    id: i64,
    date_created: String,
    uri: String,
    token: String,
    file_size: i64,
    requires_auth: bool,
    status: i64,
    file_name: String,
}

fn push_urls(vect: &mut Vec<String>, data: &Vec<Ticket>)
{
    // TODO: Use regex.
    for ticket in data
    {
        vect.push(ticket.uri.to_string());
    }
}

fn open_read_database() -> Result<Vec<Ticket>, i32>
{
    let connection = sqlite::open("../../Seven48/db.sqlite3").unwrap();
    let mut statement = connection.prepare("select * from uploader_ticket").unwrap();
    let mut tickets: Vec<Ticket> = Vec::new();
    let mut i = 0;
    while let Ok(State::Row) = statement.next() 
    {
        //println!("{i}: {} / {}", statement.read::<String, _>("id").unwrap(), statement.read::<String, _>("download_uri").unwrap());
        i += 1;
        let ticket = Ticket
        {
            id: statement.read::<i64, _>("id").unwrap(),
            date_created: statement.read::<String, _>("date_created").unwrap(),
            uri: statement.read::<String, _>("download_uri").unwrap(),
            token: statement.read::<String, _>("ticket_token").unwrap(),
            file_size: statement.read::<i64, _>("file_size").unwrap(),
            requires_auth: statement.read::<i64, _>("requires_auth").unwrap() == 1,
            status: statement.read::<i64, _>("is_done").unwrap(),
            file_name: statement.read::<String, _>("file_name").unwrap()
        };
        tickets.push(ticket);
    }

    if i == 0
    {
        return Err(0);
    }

    return Ok(tickets);
}

async fn download(url: &String, progress_spinner: &ProgressBar)
{
    let now = Instant::now();

    // TODO: Handle errors like 404 and 405.
    let client = reqwest::Client::new();
    let mut size = 0;
    let mut res = client.get(url).send().await.unwrap();

    // Create the file.
    let url_path = String::from(Url::parse(url).unwrap().path());
    let file_name = Path::new(&url_path);
    let path = Path::new("./downloads/").join(file_name.file_name().unwrap());
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", path.display(), why),
        Ok(file) => file,
    };
    
    loop
    {
        let chunk = res.chunk().await;
        if chunk.is_err()
        {
            break;
        }

        let opt = chunk.unwrap();
        if opt.is_none()
        {
            break;
        }
        
        let bs = opt.expect("Could not unwrap bytes from chunk [Option].");
        file.write_all(&bs).expect("Failed to write buffer into file.");
        size += bs.len();
        
        //progress_spinner.set_position(bs.len().try_into().unwrap());
        progress_spinner.inc(bs.len().try_into().unwrap());
        progress_spinner.set_message(format!("'{url}' : {}", console::pretty_print(size.try_into().unwrap_or(0))));
    }

    progress_spinner.finish_with_message(format!("Downloaded {url} in {}s", now.elapsed().as_secs()));
}

async fn run_download()
{
    println!("batcher in Rust.");

    let tickets = open_read_database().unwrap();
    let mut vector: Vec<String> = Vec::new();
    push_urls(&mut vector, &tickets);

    for ele in tickets
    {
        println!("- id: {}", ele.id);
        println!("- token: {}", ele.token);
        println!("- uri: {}", ele.uri);
        println!("- status: {}", ele.status);
        println!("- date created: {}", ele.date_created);
        println!("- file name: {}", ele.file_name);
        println!("- file size: {}", ele.file_size);
        println!("- requires auth (1 or 0): {}\n", ele.requires_auth);
    }

    let multi_progress = &Arc::new(MultiProgress::new());
    let download_futures = vector.iter().map(|value| async move {
        let pb = multi_progress.add(indicatif::ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::with_template("{spinner:.blue} [{elapsed_precise}] {msg} ({bytes_per_sec})")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]));
        download(value, &pb).await;
    });

    futures::future::join_all(download_futures).await;
}

fn run_prime(n: usize)
{
    let primes = prime::atkin(n);
    for prime in primes
    {
        println!("{prime}");
    }
}

fn run_primes(thread_count: i32, range: Range<usize>)
{
    let mut array = Vec::<usize>::with_capacity(5000);
    let mut i = 0;
    let mut generator = rand::thread_rng();
    let uniform = rand::distributions::Uniform::from(range);
    while i < 20
    {
        array.push(uniform.sample(&mut generator));
        i += 1;
    }

    for n in &array
    {
        print!("{n}, ");
    }
    print!("\n");

    let now = Instant::now();

    let primes = Arc::new(Mutex::new(Vec::<i64>::new()));
    let clone = Arc::clone(&primes);
    let mut processor = Processor::new(thread_count);
    processor.setup_queue(&array, move |value| {
        let now = Instant::now();
        let mut atkin_primes = prime::atkin(value);
        println!("\tComputed {} primes in {}s", atkin_primes.len(), now.elapsed().as_secs());
        clone.lock().unwrap().append(&mut atkin_primes);
    });

    let a = primes.lock().unwrap();
    for prime in a.iter()
    {
        print!("{prime}, ");
    }
    println!("Computed {} primes in {}s", a.len(), now.elapsed().as_secs());
    
}

async fn entry() -> Result<(), ()>
{
    let mut args = env::args();
    _ = args.next();
    let command = args.next().unwrap_or_default();
    
    match command.as_str()
    {
        "prime" => 
        {
            let n: usize = args.next().unwrap_or("10".to_string()).parse().unwrap();
            run_prime(n);
        }
        "primes" => 
        {
            let n: i32 = args.next().unwrap_or("7".to_string()).parse().unwrap();
            let lower: usize = args.next().unwrap_or("1000".to_string()).parse().unwrap();
            let higher: usize = args.next().unwrap_or("1000000".to_string()).parse().unwrap();
            println!("n: {n}");
            run_primes(n, lower..higher);
        }
        "batch" => 
        {
            run_download().await
        }
        _ => 
        {
            return Err(());
        }
    }
    
    return Ok(());
}

#[tokio::main]
async fn main() -> ExitCode
{
    match entry().await
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE
    }
}
