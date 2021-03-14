use crate::{leaderboard, Dungeon, LeaderboardEntry};
use bincode::config::DefaultOptions;
use bincode::Options;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::RwLock;

pub const UPLOAD_MAGIC_STRING: &str = "BEGIN THE MINING LOG";
pub const DOWNLOAD_MAGIC_STRING: &str = "GIVE ME LEADERBOARDS";

lazy_static::lazy_static! {
    static ref LEADERBOARD_ENTRIES: RwLock<Vec<u8>> = RwLock::new(Options::serialize(DefaultOptions::new(), &Vec::<LeaderboardEntry>::new()).unwrap());
}

/// This starts up a TCP server on 0.0.0.0:5852, listening for
/// incoming leaderboard submissions.
///
/// Submissions over 1MB are declined and the connection is dropped.
///
/// The runs are simulated to get the final statistics, and then
/// discarded.
///
/// The final statistics are stored in the working directory, in a
/// file called `mercury-leaderboards.bin`.
pub fn serve() {
    log::info!("Starting up leaderboard server on 0.0.0.0:8582...");
    let listener = TcpListener::bind("0.0.0.0:8582").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            std::thread::spawn(move || {
                log::debug!("Client connected: {:?}", stream);

                let mut magic_string = [0; UPLOAD_MAGIC_STRING.len()];
                if let Err(err) = stream.read_exact(&mut magic_string) {
                    log::debug!("Failed to read magic string: {}", err);
                    let _ = stream.write(b"Magic string missing.");
                    return;
                }
                if UPLOAD_MAGIC_STRING.as_bytes() == magic_string {
                    log::debug!("Client wants to submit a new run, listening for a name.");
                    handle_upload(stream);
                } else if DOWNLOAD_MAGIC_STRING.as_bytes() == magic_string {
                    log::debug!("Client wants the leaderboards, sending them over.");
                    handle_download(stream);
                } else {
                    log::debug!("Client did not start with a valid string of bytes, dropping connection.");
                    let _ = stream.write(b"Wrong magic string.");
                    return;
                }
            });
        }
    }
}

fn handle_download(mut stream: TcpStream) {
    match LEADERBOARD_ENTRIES.read() {
        Ok(data) => match stream.write_all(&data) {
            Ok(_) => log::debug!("> Done."),
            Err(err) => log::debug!("> Error writing the leaderboard data to the client: {}", err),
        },
        Err(err) => log::debug!("> Error locking the leaderboard data for sending: {}", err),
    }
}

fn handle_upload(mut stream: TcpStream) {
    let mut name_bytes = [0; 5];
    if let Err(err) = stream.read_exact(&mut name_bytes) {
        log::debug!("> Failed to read name: {}", err);
        let _ = stream.write(b"Name missing.");
        return;
    }
    let name = if name_bytes[0] as char == '>'
        && leaderboard::valid_name_character(name_bytes[1] as char)
        && leaderboard::valid_name_character(name_bytes[2] as char)
        && leaderboard::valid_name_character(name_bytes[3] as char)
        && name_bytes[4] as char == '<'
    {
        let name = [name_bytes[1] as char, name_bytes[2] as char, name_bytes[3] as char];
        log::debug!("> Name {}{}{} is ok, listening for the run.", name[0], name[1], name[2]);
        name
    } else {
        log::debug!("> Invalid name format.");
        let _ = stream.write(b"Invalid name.");
        return;
    };

    let mut run_bytes = Vec::with_capacity(10_000);
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Ok(n) => {
                run_bytes.extend_from_slice(&buffer[..n]);
                if run_bytes.len() > 1_000_000 {
                    log::debug!("> Client tried to upload over 1MB of run data, dropping connection.");
                    let _ = stream.write(b"No spam!");
                    return;
                }
            }
            Err(err) => {
                log::error!("> Error while receiving run: {}", err);
                let _ = stream.write(b"Connection issue.");
                return;
            }
        }
    }

    log::debug!("> Run received, deserializing.");
    match Dungeon::from_bytes(&run_bytes) {
        Ok(dungeon) => {
            log::debug!("> Deserialization successful, updating leaderboards.");
            log::debug!(
                "> Name: {:?}, {} treasure, {} rounds.",
                name,
                dungeon.treasure(),
                dungeon.round()
            );

            // TODO: Check that the run is either game over or finished
            let new_entry = LeaderboardEntry {
                name,
                treasure: dungeon.treasure(),
                rounds: if dungeon.is_game_over() {
                    None
                } else if dungeon.final_treasure_found() {
                    Some(dungeon.round())
                } else {
                    log::debug!("> Got a run that hadn't ended, dropping.");
                    let _ = stream.write(b"No early exits!");
                    return;
                },
                size: run_bytes.len(),
            };

            match LEADERBOARD_ENTRIES.write() {
                Ok(mut entries_bytes) => {
                    let mut entries: Vec<LeaderboardEntry> =
                        Options::deserialize(DefaultOptions::new(), &entries_bytes).unwrap();
                    log::debug!("> Writing: {:?}", new_entry);
                    entries.push(new_entry);
                    *entries_bytes = Options::serialize(DefaultOptions::new(), &entries).unwrap();
                }
                Err(err) => {
                    log::error!("> Error locking the leaderboard array: {}", err);
                }
            }

            let _ = stream.write(b"OK.");
        }

        Err(err) => {
            log::debug!("> Deserialization error: {}", err);
            let _ = stream.write(b"Version too old.");
            return;
        }
    }
}
