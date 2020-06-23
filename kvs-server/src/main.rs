#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate simplelog;
use kvs::{Command, KvStore, KvsEngine, Result};
use simplelog::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, thread};
use structopt::StructOpt;
#[derive(StructOpt, Debug)]
#[structopt(name = "kvs server", about = "the kvs server instance")]
struct Cli {
    #[structopt(
        value_name = "addr",
        help = "The remote IP address IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,

    #[structopt(
        value_name = "engine",
        help = "Engine to use for stuff",
        default_value = "kvs"
    )]
    engine: String,
}

fn read_cmd(stream: TcpStream) -> Result<()> {
    println!("Incoming connection from: {:?}", stream.peer_addr());
    info!("Incoming connection from: {:?}", stream.peer_addr());

    let mut data = Vec::new();
    let mut buffer = BufReader::new(stream);

    data.clear();
    let bytes_read = buffer.read_until(b'\n', &mut data).unwrap();
    if bytes_read == 0 {
        return Ok(());
    }
    let input: Command = serde_json::from_slice(&data).unwrap();
    // let value: Command = input;
    // println!("{:?}{:?}", &value, &value);

    // println!("Request: {:?}", &input);
    // process_cmd(input);
    match input {
        Command::Set { key, value } => {
            // println!("{} - {}", key, value);
            let path = env::current_dir().unwrap();
            let mut store = KvStore::open(path).unwrap();
            store.set(key, value).expect("set failed");
            // warn!("{}", store);
            // let key = input::Set::key;
            // warn!("current store handle {:#?}", &key);
            // store
            //     .set(input::Set::key, input::Set::value)
            //     .expect("set failed");
            // drop(store);
        }
        Command::Get { key } => {
            let path = env::current_dir().unwrap();
            let mut store = KvStore::open(path.to_path_buf()).unwrap();
            warn!("Looking for the value of: {:?}", &key);
            // println!("data {:?}", &store);
            let resutls = store.get(key).expect("No key found");
            println!("value for key: {:?}", &resutls);
            // drop(store);
        }
        Command::Rm { key } => {
            let path = env::current_dir().unwrap();
            let mut store = KvStore::open(path.to_path_buf()).unwrap();
            warn!("This key *{}* shall be removed, if available", &key);
            store.remove(key).expect("Del failed");
            // drop(store);
        } // _ => panic!("dead dead, wtf did you do?"),
    }
    Ok(())
}

fn process_cmd(_input: Command) -> Result<()> {
    // match input {
    //     Command::Set { key, value } => {
    //         // println!("{} - {}", key, value);
    //         // let (key, value) = Command::Set { key, value };
    //         let mut store = KvStore::open(env::current_dir().unwrap()).unwrap();
    //         store.set(key, value).expect("set failed");
    //         // warn!("{}", store);
    //         // let key = input::Set::key;
    //         // warn!("current store handle {:#?}", &key);
    //         // store
    //         //     .set(input::Set::key, input::Set::value)
    //         //     .expect("set failed");
    //         // drop(store);
    //     }
    //     Command::Get { key } => {
    //         let path = env::current_dir().unwrap();
    //         let mut store = KvStore::open(path).unwrap();
    //         warn!("Looking for the value of: {:?}", &key);
    //         // println!("data {:?}", &store);
    //         store.get(key).expect("get failed");

    //         // drop(store);
    //     }
    //     Command::Rm { key } => {
    //         let mut store = KvStore::open(env::current_dir().unwrap())?;
    //         warn!("This key *{}* shall be removed, if available", &key);
    //         store.remove(key);
    //         // drop(store);
    //     }
    //     _ => panic!("dead dead, wtf did you do?"),
    // }
    Ok(())
}

fn respond() {}

fn main() {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Warn, Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("conn.log").unwrap(),
        ),
    ])
    .unwrap();
    let opt = Cli::from_args();
    warn!("Server version is: {:?}", env!("CARGO_PKG_VERSION"));
    warn!("Configuration: {:?}", &opt);

    println!("{:?}", opt.addr);
    // match opt {
    //     Opt::addr => {
    //         println!("{:?}". )
    //     }
    // }
    // let addr = opt.addr;
    let listener = TcpListener::bind("127.0.0.1:4000").expect("Could not bind");
    for stream in listener.incoming() {
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(stream) => {
                thread::spawn(move || {
                    read_cmd(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}
