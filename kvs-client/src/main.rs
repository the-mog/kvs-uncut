#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate serde_derive;
extern crate simplelog;
use clap::{App, Arg};
use kvs::{Command, Result};
use simplelog::*;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process;

// #[derive(Serialize, Deserialize, Debug)]
// pub enum Command {
//     Set { key: String, value: String },
//     Get { key: String },
//     Rm { key: String },
// }

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Warn, Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("kvs_rt.log").unwrap(),
        ),
    ])
    .unwrap();
    let matches = App::new("kvs")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            App::new("set")
                .about("The string key and value <key> <value>")
                .args(&[
                    Arg::with_name("key")
                        .value_name("KEY")
                        .long("key")
                        .short('k')
                        .about("the key")
                        .takes_value(true),
                    Arg::with_name("value")
                        .value_name("VALUE")
                        .long("value")
                        .short('v')
                        .about("the value")
                        .takes_value(true),
                ]),
        )
        .subcommand(
            App::new("get").about("Retrieve the value of the key").arg(
                Arg::with_name("get")
                    .value_name("GET")
                    // .long("get")
                    // .short('g')
                    // .about("Retrieve the value of the key")
                    .takes_value(true),
            ),
        )
        .subcommand(
            App::new("rm").about("Remove the key").arg(
                Arg::with_name("rm")
                    .value_name("DELETE")
                    // .long("rm")
                    // .short('d')
                    .about("Remove the key")
                    .takes_value(true),
            ),
        )
        .arg(
            Arg::with_name("kvs")
                .value_name("kvs app")
                .about("kvs app for rustliungs")
                .takes_value(false),
        )
        .get_matches();

    // if let Some(set_subs) = matches.subcommand_matches("set") {
    //     if set_subs.is_present("key") {
    //         let key = set_subs.value_of("key");
    //         let val = set_subs.value_of("value");
    //         let
    //         eprintln!("Setting key");
    //         println!("set: {:?}{:?}", &key, &val);

    //         process::exit(1);
    //     }
    // }
    // if matches.is_present("get") {
    //     eprintln!("Key not found");
    //     process::exit(1);
    // }
    // if matches.is_present("rm") {
    //     eprintln!("Key not found");
    //     process::exit(1);
    // }
    // if !matches.is_present("kvs") {
    //     eprintln!("Key not found");
    //     process::exit(1);
    // }
    match matches.subcommand() {
        ("set", Some(set_subs)) => {
            if set_subs.is_present("key") {
                let key = set_subs.value_of("key").unwrap().to_string();
                let value = set_subs.value_of("value").unwrap().to_string();
                // eprintln!("Saving key");
                // warn!("{:?}, {:?}", &key, &value);
                // let mut store = KvStore::open(env::current_dir()?);
                let setcmd = Command::set(key, value);
                let mut stream =
                    TcpStream::connect("127.0.0.1:4000").expect("could connect to the server");
                stream
                    .write_all(serde_json::to_string(&setcmd).unwrap().as_bytes())
                    .expect("Failed to write to server");
                stream.write_all(b"\n").expect("Failed to write to server");
                // let mut buf = [0; 10];
                // stream.peek(&mut buf).expect("peek failed");
                // let mut store = KvStore::open()?;
                // // warn!("current store handle {:?}", &store);
                // store.set(key, value).expect("set failed");
                // println!("Response: {}", String::from_utf8_lossy(&stream));
                process::exit(1);
            }
        }
        ("get", Some(get_subs)) => {
            if get_subs.is_present("get") {
                let key = get_subs.value_of("get").unwrap().to_string();
                // error!("key is {:?}", key);
                //     KvStore::open(std::path::PathBuf::from("./log"));
                //     process::exit(1);
                // } else {
                //     error!("try running with -h");
                // }
                let setcmd = Command::get(key);
                let mut stream =
                    TcpStream::connect("127.0.0.1:4000").expect("could connect to the server");
                stream
                    .write_all(serde_json::to_string(&setcmd).unwrap().as_bytes())
                    .expect("Failed to write to server");
                stream.write_all(b"\n").expect("Failed to write to server");
                process::exit(1);
            }
        }
        ("rm", Some(rm_subs)) => {
            if rm_subs.is_present("rm") {
                let key = rm_subs.value_of("rm").unwrap().to_string();
                // error!("key is {:?}", key);
                //     KvStore::open(std::path::PathBuf::from("./log"));
                //     process::exit(1);
                // } else {
                //     error!("try running with -h");
                // }
                let setcmd = Command::rm(key);
                let mut stream =
                    TcpStream::connect("127.0.0.1:4000").expect("could connect to the server");
                stream
                    .write_all(serde_json::to_string(&setcmd).unwrap().as_bytes())
                    .expect("Failed to write to server");
                stream.write_all(b"\n").expect("Failed to write to server");
                process::exit(1);
            }
        }
        _ => {
            warn!("try running with -h");
            //panic!("crash coming near you...")
            process::exit(1);
        }
    }
    Ok(())
}
