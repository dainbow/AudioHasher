use blake2::{Blake2s256, Digest};
use clap::{Parser, Subcommand};
use file_hashing::get_hash_file;
use serde::{Deserialize, Serialize};
use skyscraper::xpath::XpathItemTree;
use skyscraper::{html, xpath};
use std::io::{self, prelude::*};
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
        #[arg(default_value = "./answers.html")]
        html_path: String,
    },
    Solve {
        #[arg(default_value = "./answers.json")]
        dict_path: String,
        #[arg(default_value = ".")]
        question_path: String,
    },
}

#[derive(Serialize, Deserialize, Default)]
struct Answers(HashMap<String, Vec<(String, String)>>);

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Generate { html_path } => {
            let mut answers = Answers::default();
            let mut file = File::open(html_path).unwrap();
            let mut contents = String::new();
            let _ = file.read_to_string(&mut contents);

            let document = html::parse(&contents).unwrap();
            let xpath_item_tree = XpathItemTree::from(&document);

            let xpath =
                xpath::parse("/html/body/div[2]/div[2]/div/div/section[1]/div[1]/form/div/div")
                    .unwrap();
            let questions_amount = xpath.apply(&xpath_item_tree).unwrap().len();

            for i in 1..questions_amount {
                let audio_path = format!("/html/body/div[2]/div[2]/div/div/section[1]/div[1]/form/div/div[{}]/div[2]/div/div[1]/div/div/div/div/div/div[4]/a", i);
                let xpath = xpath::parse(&audio_path).unwrap();
                let audio_name: String = xpath.apply(&xpath_item_tree).unwrap()[0]
                    .extract_as_node()
                    .extract_as_tree_node()
                    .data
                    .extract_as_element_node()
                    .get_attribute("href")
                    .unwrap()
                    .chars()
                    .take_while(|c| *c != '?')
                    .collect();
                let audio_name = audio_name
                    .split('/')
                    .last()
                    .unwrap()
                    .to_owned()
                    .replace("%20", " ");
                let mut hash = Blake2s256::new();

                let audio_hash = get_hash_file(&audio_name, &mut hash).unwrap();
                let prev_vec = answers.0.entry(audio_hash).or_insert(vec![]);

                let xpath =
                xpath::parse(&format!("/html/body/div[2]/div[2]/div/div/section[1]/div[1]/form/div/div[{}]/div[2]/div/div[2]/p", i))
                    .unwrap();
                let answer_sections_amount = xpath.apply(&xpath_item_tree).unwrap().len();

                for section in 1..=answer_sections_amount {
                    let xpath =
                        xpath::parse(&format!("/html/body/div[2]/div[2]/div/div/section[1]/div[1]/form/div/div[{}]/div[2]/div/div[2]/p[{}]/span", i, section))
                            .unwrap();
                    let answers_amount = xpath.apply(&xpath_item_tree).unwrap().len();
                    for ans in 1..=answers_amount {
                        let answers_path = format!("/html/body/div[2]/div[2]/div/div/section[1]/div[1]/form/div/div[{}]/div[2]/div/div[2]/p[{}]/span[{}]", i, section, ans);
                        let xpath = xpath::parse(&answers_path)
                            .unwrap()
                            .apply(&xpath_item_tree)
                            .unwrap();
                        let res = xpath[0].extract_as_node().extract_as_tree_node();
                        if let Some(answer_type) =
                            res.data.extract_as_element_node().get_attribute("class")
                        {
                            prev_vec.push((
                                if answer_type.contains("incorrect") {
                                    "WRONG".to_owned()
                                } else {
                                    "OK".to_owned()
                                },
                                res.text(&xpath_item_tree)
                                    .unwrap()
                                    .to_owned()
                                    .replace("&nbsp;", ""),
                            ));
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
