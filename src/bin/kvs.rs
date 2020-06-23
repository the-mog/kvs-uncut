#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;
use clap::{App, Arg};
use kvs::Result;
use kvs::{KvStore, KvsEngine};
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, process};

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Warn, Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
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
    warn!("Server version");
    info!("Configuration");
    match matches.subcommand() {
        ("set", Some(set_subs)) => {
            if set_subs.is_present("key") {
                let key = set_subs.value_of("key").unwrap().to_string();
                let value = set_subs.value_of("value").unwrap().to_string();
                // eprintln!("Saving key");
                // warn!("{:?}, {:?}", &key, &value);
                let mut store = KvStore::open(env::current_dir().unwrap()).unwrap();
                // let mut store = KvStore::open().unwrap();
                // warn!("current store handle {:?}", &store);
                // warn!("Index2: {:?}", &store.index);
                store.set(key, value).expect("set failed");
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
                let path = env::current_dir().unwrap();
                let mut store = KvStore::open(path).unwrap();
                if let Some(value) = store.get(key)? {
                    println!("Value of key is: {}", value);
                } else {
                    println!("sorry dude, no key found");
                }
            } else {
                warn!("invalid input, try running with -h")
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
                let mut store = KvStore::open(env::current_dir().unwrap())?;
                store.remove(key)?;
            } else {
                println!("sorry dude, some went horribly wrong");
            }
        }
        _ => warn!("try running with -h"), //panic!("crash coming near you...")
    }
    Ok(())
}
