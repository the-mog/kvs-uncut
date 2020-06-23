extern crate failure;
extern crate serde;
extern crate toml;

extern crate failure_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate simplelog;
use failure::Fail;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{read_dir, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, KvsError>;
#[derive(Fail, Debug)]
pub enum KvsError {
    /// Removing non-existent key error
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

impl Command {
    pub fn set(key: String, value: String) -> Command {
        // warn!("set command enum");
        Command::Set { key, value }
    }
    pub fn get(key: String) -> Command {
        Command::Get { key }
    }
    pub fn rm(key: String) -> Command {
        Command::Rm { key }
    }
}

pub struct KvsClient {}
pub struct KvsServer {}

pub trait KvsEngine {
    fn open(path: PathBuf) -> Result<KvStore>;
    ///set the value of a string key to a string
    fn set(&mut self, key: String, value: String) -> Result<()>;

    fn get(&mut self, key: String) -> Result<Option<String>>;

    fn remove(&mut self, key: String) -> Result<()>;
}
pub struct SledKvsEngine {}
pub struct KvStore {
    /// Directory for the log and other data
    path: PathBuf,
    /// The log reader
    readers: HashMap<u64, BufReaderWithPos<File>>,
    /// The log writer
    writer: BufWriterWithPos<File>,
    /// The in-memory index from key to log pointer
    index: BTreeMap<String, CommandPos>,
    current_index: u64,
}

impl KvsEngine for KvStore {
    ///create a new key value pair
    fn open(path: PathBuf) -> Result<KvStore> {
        let index = BTreeMap::new();
        let path = path;
        let mut readers = HashMap::new();

        let list = list_log_files(&path).unwrap();
        let current_index = list.last().unwrap_or(&0) + 1;

        for &idx in &list {
            let read =
                BufReaderWithPos::new(File::open(path.join(format!("{}.log", idx))).unwrap())
                    .unwrap();
            //<idx, path>
            readers.insert(idx, read);
        }
        // warn!(
        //     "Writing the following to log file path:{:?} curr_index{}",
        //     &path, &current_index
        // );
        let writer = new_log_file(&path, current_index, &mut readers).unwrap();

        Ok(KvStore {
            index,
            path,
            readers,
            writer,
            current_index,
        })
    }
    ///set the value of a string key to a string
    fn set(&mut self, key: String, value: String) -> Result<()> {
        // self.data.insert(&key, &value);
        // let mut file = File::create("./log").unwrap();
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        // let idx = self.current_index;
        // let setcmd = format!("{:?} {:?}", &idx, &cmd);
        // warn!("Serializing cmd {:?} at this index {:?}", &cmd, &idx);
        serde_json::to_writer(&mut self.writer, &cmd).expect("serialisation failed");
        if let Command::Set { key, .. } = cmd {
            self.index
                .insert(key, (self.current_index, pos..self.writer.pos).into());
        }
        warn!(
            "In-memory data: Index: {:?}, self.writer.pos {:?}. pos {}.",
            &self.index, self.writer.pos, pos
        );
        Ok(())
    }
    ///get the string value of the string key.
    /// if the key does not exist, return None
    fn get(&mut self, fkey: String) -> Result<Option<String>> {
        // Ok(self.data.get(&key).cloned())
        // if Some(key) = self.da
        // warn!("Db {:?} key {:?}", &self.index, &key);
        // let getpos = self.readers.get(&self.current_index).expect("No key found");
        // warn!("Looking for value of *{:?}*", &key);
        // warn!("Current log file index {:?}", &getpos.current_gen);
        let list = list_log_files(&self.path).unwrap();
        let lastfile = list.last().unwrap() - 1;
        warn!("last file index {}", &lastfile);
        // let reader = self
        //     .readers
        //     .get_mut(&lastfile)
        //     .expect("file not found");
        // // let file = format!("{:?}.log",&self.current_index);
        // warn!("wassup {:?}", &getpos.len);

        // let mut read = BufReaderWithPos::new(
        //     File::open(&self.path.join(format!("{}.log", lastfile))).unwrap(),
        // )
        // .unwrap();
        let file = File::open(format!("{}.log", &lastfile)).unwrap();
        let reader = BufReader::new(file);
        let data: Command = serde_json::from_reader(reader).expect("failed to parse the file");
        warn!("data is: {:?}", &data);
        match data {
            Command::Set { key, value } => {
                // warn!("key {} fkey {} value {}", &key, &fkey, &value);
                // if key == fkey {
                //     return Ok(Some(value));
                // } else {
                //     return Err(KvsError::UnexpectedCommandType);
                // }
                // let results = match fkey {
                //     key => Ok(Some(value)),
                //     _ => Err(KvsError::UnexpectedCommandType),
                // };
                if key == fkey {
                    println!("shiiiit, we got it..");
                    return Ok(Some(value));
                } else {
                    println!("mehhh, cant find the key.");
                    // return Err(KvsError::KeyNotFound)
                }
                // return results
            }
            Command::Rm { key } => {
                if key == fkey {
                    println!("shiiiit, that key is marked for deleted, so no data for you");
                } else {
                    println!("mehhh, cant find the key.");
                    // return Err(KvsError::KeyNotFound)
                }
            }
            _ => panic!("failed at coding"),
        }

        // return Ok(Some(value));
        //     } else {
        //         return Err(KvsError::UnexpectedCommandType);
        //     }
        // } else {
        //     Ok(None)
        // read.seek(SeekFrom::Start(read.pos)).unwrap();
        // let cmdreader = read.take(self.index.len);
        // warn!("File to read has {:?} bytes", &read.len);
        // if let Command::Set { value, .. } = serde_json::from_reader(cmdreader).unwrap() {
        //     return Ok(Some(value));
        // } else {
        //     return Err(KvsError::UnexpectedCommandType);
        // }

        // if let Some(getpos) = self.index.get(&key) {
        //     warn!("Looking for value of *{:?}*", &key);
        //     warn!("Current log file index {:?}", &getpos.current_gen);
        //     let read = self
        //         .readers
        //         .get_mut(&getpos.current_gen)
        //         .expect("file not found");

        //     warn!("wassup {:?}", &getpos.len);

        //     read.seek(SeekFrom::Start(getpos.pos)).unwrap();
        //     let cmdreader = read.take(getpos.len);
        //     warn!("File to read has {:?} bytes", &getpos.len);
        //     if let Command::Set { value, .. } = serde_json::from_reader(cmdreader).unwrap() {
        //         return Ok(Some(value));
        //     } else {
        //         return Err(KvsError::UnexpectedCommandType);
        //     }
        // } else {
        //     Ok(None)
        // }
        Ok(None)
    }
    ///remove a given key
    fn remove(&mut self, fkey: String) -> Result<()> {
        // let del = self.data.remove(&key);
        // println!("Key removal status: {:?}", del);
        // Ok(())
        // if self.index.contains_key(&key) {
        //     let rmcmd = Command::rm(key);
        //     serde_json::to_writer(&mut self.writer, &rmcmd).unwrap();

        //     if let Command::Rm { key } = rmcmd {
        //         warn!("Index before del {:?}", &self.index);
        //         self.index.remove(&key).expect("invalid/empty key");
        //         warn!("Index after del {:?}", &self.index);
        //     }
        //     Ok(())
        // } else {
        //     Err(KvsError::KeyNotFound)
        // }
        let list = list_log_files(&self.path).unwrap();
        // since get also creates an empty file..got 2 step back
        let lastfile = list.last().unwrap() - 2;
        warn!("last file {}", &lastfile);
        // let mut read = BufReaderWithPos::new(
        //     File::open(&self.path.join(format!("{}.log", lastfile))).unwrap(),
        // )
        // .unwrap();
        let file = File::open(format!("{}.log", &lastfile)).unwrap();
        let reader = BufReader::new(file);
        let data: Command = serde_json::from_reader(reader).expect("failed to parse the file");
        warn!("data is {:?}", &data);
        match data {
            Command::Set { key, value } => {
                // warn!("key {} fkey {} value {}", &key, &fkey, &value);
                // if key == fkey {
                //     return Ok(Some(value));
                // } else {
                //     return Err(KvsError::UnexpectedCommandType);
                // }
                // let results = match fkey {
                //     key => Ok(Some(value)),
                //     _ => Err(KvsError::UnexpectedCommandType),
                // };
                if key == fkey {
                    println!("shiiiit, we found your file to be removed");
                    let cmd = Command::rm(key);
                    serde_json::to_writer(&mut self.writer, &cmd).expect("serialisation failed");
                    // warn!("Data to be removed written to {:?}", &self.writer);
                    Ok(())
                } else {
                    println!("mehhh");
                    return Err(KvsError::KeyNotFound);
                }
                // return results
            }
            Command::Rm { key } => {
                println!("{} already marked for removal.", key);
                Ok(())
            }
            _ => panic!("failed at coding"),
        }
    }
}

// pub fn open(path: PathBuf) -> Result<()> {
//     let lines = BufReader::new(File::open(path).unwrap())
//         .lines()
//         .collect::<Vec<_>>();
//     if !lines.is_empty() {
//         println!("empty file");
//     }
//     let ss = match String::from_utf8(lines) {
//         Ok(v) => v,
//         Err(e) => panic!("non-utf8"),
//     };
//     println!("line is: {:?}", lines[0]);

// // Ok((KvStore))
// let mut file = File::open(path).unwrap();
// let mut contents = Vec::new();
// file.read_to_end(&muontents).unwrap();

// let map = unsafe { Mmap::map(&file).unwrap() };
// println!("{:?}", map);
// let ss = match String::from_utf8(map) {
//     Ok(v) => v,
//     Err(e) => panic!("invalid")
// };
// let random_indexes = [0, 1, 2, 19, 22, 10, 11, 29];
// let random_bytes: Vec<u8> = random_indexes.iter()
//     // .map(|&idx| map[idx])
//     // .collect();
//     println!("{:?}", &ss);

//     Ok(())
// }

////Dealing with log files
///
fn list_log_files(path: &PathBuf) -> Result<Vec<u64>> {
    let mut list: Vec<u64> = read_dir(&path)
        .unwrap()
        .flat_map(|res| -> Result<_> { Ok(res.unwrap().path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    list.sort_unstable();
    warn!("Log file count: {:?}", &list);
    Ok(list)
}

fn new_log_file(
    path: &Path,
    index: u64,
    reader: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = path.join(format!("{}.log", index));

    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)
            .unwrap(),
    )
    .unwrap();
    warn!(
        "Writng the following to the file: path:{:?}, index:{:?}",
        &path, &index
    );
    reader.insert(
        index,
        BufReaderWithPos::new(File::open(&path).unwrap()).unwrap(),
    );
    Ok(writer)
}

#[derive(Debug)]
struct CommandPos {
    current_gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((current_gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            current_gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0)).unwrap();
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        //returns how many bytes were returned
        let len = self.writer.write(buf).unwrap();
        // warn!("len and pos before: {:?} {:?}", &len, &self.pos);
        self.pos += len as u64;
        // warn!("len and pos after: {:?} {:?}", &len, self.pos);
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos).unwrap();
        Ok(self.pos)
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0)).unwrap();
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos += self.reader.seek(pos)?;
        Ok(self.pos)
    }
}
