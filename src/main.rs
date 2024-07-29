use aus::AudioError;
use glob::glob;
use threadpool::ThreadPool;

/// Finds all audio files that match a search pattern.
/// For each file, returns a tuple that has the full path, directory, and file name without extension.
fn find_audio(directory: &str) -> Vec<(String, String, String)> {
    let mut file_paths: Vec<(String, String, String)> = Vec::new();
    let extensions = vec!["aif", "aiff", "mp3", "flac", "ogg", "aac", "m4a", "wma"];
    for extension in extensions {
        let entries = glob(&format!("{}/**/*.{}", directory, extension));
        match entries {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            let file_name = match path.file_stem() {
                                Some(x) => match x.to_str() {
                                    Some(y) => String::from(y),
                                    None => String::from("")
                                },
                                None => String::from("")
                            };
                            let mut parent = path.clone();
                            parent.pop();
                            let parent = match parent.to_str() {
                                Some(x) => String::from(x),
                                None => String::from("")
                            };
                            let path = match path.to_str() {
                                Some(x) => String::from(x),
                                None => String::from("")
                            };
                            file_paths.push((path, parent, file_name));
                        },
                        Err(_) => ()
                    };
                }
            },
            Err(_) => ()
        }
    }
    file_paths
}

/// Processes all of the files in a file vector and converts them to WAV
fn process(files: &Vec<(String, String, String)>, max_num_threads: usize) {
    let max_available_threads = match std::thread::available_parallelism() {
        Ok(x) => x.get(),
        Err(_) => 1
    };

    let num_threads = if max_num_threads < 1 {
        max_available_threads
    } else {
        usize::min(max_available_threads, max_num_threads)
    };

    let pool = ThreadPool::new(num_threads);
    for file_tup in files.iter() {
        let file = file_tup.0.clone();
        let dir = file_tup.1.clone();
        let name = file_tup.2.clone();
        pool.execute(move || {
            println!("File: {}", file);
            match aus::read(&file) {
                Ok(mut audio) => {
                    if audio.bits_per_sample == 0 {
                        audio.bits_per_sample = 16;
                        audio.audio_format = aus::AudioFormat::S16;
                    }
                    if audio.sample_rate == 0 {
                        audio.sample_rate = 44100;
                    }
                    let new_file_name = format!("{}/{}.wav", dir, name);
                    match aus::write(&new_file_name, &audio) {
                        Ok(_) => (),
                        Err(err) => match err {
                            AudioError::FileCorrupt => println!("Error writing file {}: The file was corrupt.", new_file_name),
                            AudioError::FileInaccessible(msg) => println!("Error writing file {}: The file was inaccessible ({}).", new_file_name, msg),
                            AudioError::NumChannels(msg) => println!("Error writing file {}: The number of channels was wrong ({}).", new_file_name, msg),
                            AudioError::NumFrames(msg) => println!("Error writing file {}: The number of frames was wrong ({}).", new_file_name, msg),
                            AudioError::SampleValueOutOfRange(msg) => println!("Error writing file {}: A sampel value was out of range ({}).", new_file_name, msg),
                            AudioError::WrongFormat(msg) => println!("Error writing file {}: The format was wrong ({}).", new_file_name, msg)
                        }
                    };
                },
                Err(err) => match err {
                    AudioError::FileCorrupt => println!("Error writing file {}: The file was corrupt.", file),
                    AudioError::FileInaccessible(msg) => println!("Error writing file {}: The file was inaccessible ({}).", file, msg),
                    AudioError::NumChannels(msg) => println!("Error writing file {}: The number of channels was wrong ({}).", file, msg),
                    AudioError::NumFrames(msg) => println!("Error writing file {}: The number of frames was wrong ({}).", file, msg),
                    AudioError::SampleValueOutOfRange(msg) => println!("Error writing file {}: A sampel value was out of range ({}).", file, msg),
                    AudioError::WrongFormat(msg) => println!("Error writing file {}: The format was wrong ({}).", file, msg)
                }
            }
        });
    }

    pool.join();
}

struct Args {
    folder: String,
    num_threads: usize,
    delete: bool
}

/// Validates the command line arguments
fn validate_args(args: Vec<String>) -> Option<Args> {
    if args.len() <= 6 {
        let valid_args = std::collections::HashMap::from([("-f", 1), ("--folder", 1), ("-n", 1), ("--num-threads", 1), ("-d", 1), ("--delete", 1)]);
        let mut processed_args: Args = Args{folder: String::from("."), num_threads: 0, delete: false};
        let mut i = 1;
        while i < args.len() {
            if !valid_args.contains_key(args[i].as_str()) {
                return None;
            } else {
                match args[i].as_str() {
                    "-f" | "--folder" => {
                        processed_args.folder = args[i+1].clone();
                        i += 2;
                    },
                    "-n" | "--num-threads" => {
                        processed_args.num_threads = match args[i+1].parse::<usize>() {
                            Ok(x) => x,
                            Err(_) => return None
                        };
                        i += 2;
                    },
                    "-d" | "--delete" => {
                        processed_args.delete = true;
                        i += 1;
                    }
                    _ => {
                        return None;
                    }
                }
            }
        }

        return Some(processed_args);
    } else {
        return None;
    }
}

fn main() {
    // process the arguments
    let args = match validate_args(std::env::args().collect()) {
        Some(x) => x,
        None => {
            println!("Usage:\n-f folder_name -n num_threads\nOptional: include the -d flag to delete the original files when done.");
            return;
        }
    };
    
    // convert the files
    let files = find_audio(&args.folder);
    println!("Converting {} files...", files.len());
    process(&files, args.num_threads);

    // delete the old files if asked to
    if args.delete {
        println!("Deleting original files...");
        for file in files.iter() {
            match std::fs::remove_file(file.0.as_str()) {
                Ok(_) => (),
                Err(_) => ()
            };
        }
    }
    println!("Done.");
}
