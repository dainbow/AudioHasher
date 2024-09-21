use blake2::{Blake2s256, Digest};
use clap::{Parser, Subcommand};
use file_hashing::get_hash_file;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Generate {
        #[arg(default_value = ".")]
        answers_dir: String,
    },
    Solve {
        #[arg(default_value = "./answers.json")]
        dict_path: String,
        #[arg(default_value = ".")]
        question_path: String,
    },
}

#[derive(Serialize, Deserialize, Default)]
struct Answers(HashMap<String, Vec<String>>);

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Generate { answers_dir } => {
            let path = Path::new(&answers_dir);
            let mut answers = Answers::default();

            for audio in path
                .read_dir()
                .expect("Generate subcommand should take a directory as answers_dir")
            {
                if let Ok(entry) = audio {
                    let audio_path = entry.path();
                    if let Some(file_ext) = audio_path.extension() {
                        if file_ext == "mp3" {
                            let mut hash = Blake2s256::new();

                            let audio_hash = get_hash_file(&audio_path, &mut hash).unwrap();

                            let prev_vec = answers.0.entry(audio_hash).or_insert(vec![]);
                            prev_vec.push(
                                audio_path
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy()
                                    .into_owned(),
                            );
                        }
                    }
                }
            }

            let mut file = File::create("answers.json").unwrap();
            let _ = file.write_all(serde_json::to_string(&answers).unwrap().as_bytes());
        }
        Commands::Solve {
            dict_path,
            question_path,
        } => {
            let mut file = File::open(dict_path).unwrap();
            let mut contents = String::new();
            let _ = file.read_to_string(&mut contents);

            let path = Path::new(&question_path);
            let answers: Answers = serde_json::from_str(&contents).unwrap();

            for audio in path
                .read_dir()
                .expect("Solve subcommand should take a directory as question_path")
            {
                if let Ok(entry) = audio {
                    let audio_path = entry.path();
                    if let Some(file_ext) = audio_path.extension() {
                        if file_ext == "mp3" {
                            let mut hash = Blake2s256::new();

                            let audio_hash = get_hash_file(&audio_path, &mut hash).unwrap();

                            println!(
                                "{:?} -> {:?}",
                                audio_path.file_stem().unwrap(),
                                answers.0.get(&audio_hash).unwrap_or(&vec![])
                            );
                        }
                    }
                }
            }

            let _ = file.write_all(serde_json::to_string(&answers).unwrap().as_bytes());
        }
    }
}
