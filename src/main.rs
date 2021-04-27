use serde::Deserialize;
use std::env;
use std::fmt;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), String> {
    let matches = App::new("Define")
                    .version(env!("CARGO_PKG_VERSION"))
                    .author("John Brehm (Cooljohnny3)")
                    .about("A command line dictionary application using the Free Dictionary API. (https://dictionaryapi.dev/)")
                    .arg(Arg::with_name("WORD")
                        .help("Word to search for")
                        .required(true))
                    .arg(Arg::with_name("language_code")
                        .short("l")
                        .long("language_code")
                        .value_name("CODE")
                        .help("Sets the language to search in. Supported codes (en_US, es). default=en_US"))
                    .get_matches();

    let word = matches.value_of("WORD").expect("Invalid word");
    let code = matches.value_of("language_code").unwrap_or("en_US");
    let thing = get_word(String::from(word), String::from(code)).await;
    match thing {
        Ok(def) => {
            print!("{}", &def[0]);
            Ok(())
        }
        Err(e) => {
            let err;
            if e.is_connect() {
                err = Err(String::from("Connection Error"));
            } else if e.status() == None {
                err = Err(String::from("Word not found"));
            } else {
                err = Err(String::from("Unknown Error"));
            }
            err
        },
    }
}

async fn get_word(word: String, code: String) -> Result<Vec<Word>, reqwest::Error> {
    let response: Vec<Word> = reqwest::get(format!(
        "https://api.dictionaryapi.dev/api/v2/entries/{}/{}",
        code, word
    ))
    .await?
    .json()
    .await?;
    Ok(response)
}

#[derive(Deserialize, Debug)]
struct Word {
    word: String,
    origin: Option<String>,
    phonetics: Option<Vec<Phonetic>>,
    meanings: Vec<Meaning>,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Phonetics
        let mut phonetics: String = String::new();
        if let Some(phonetics_array)  = &self.phonetics {
            if let Some(_) = phonetics_array[0].text {
                phonetics.push('[');
                phonetics.push_str(&phonetics_array[0].to_string());

                for i in 1..phonetics_array.len() {
                    phonetics.push_str(", ");
                    phonetics.push_str(&phonetics_array[i].to_string());
                }
                phonetics.push(']');
            }
        }

        // Meanings
        let mut meanings: String = String::new();
        meanings.push_str(&self.meanings[0].to_string());

        for i in 1..self.meanings.len() {
            meanings.push_str(&self.meanings[i].to_string());
        }

        write!(f, "{} {}{}{}", self.word, phonetics, self.origin.as_ref().unwrap_or(&String::from("")), meanings)
    }
}

#[derive(Deserialize, Debug)]
struct Phonetic {
    text: Option<String>,
    audio: Option<String>,
}

impl fmt::Display for Phonetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text.as_ref().unwrap_or(&String::from("")))
    }
}

#[derive(Deserialize, Debug)]
struct Meaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,
    definitions: Vec<Definition>,
}

impl fmt::Display for Meaning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut definitions: String = String::new();

        for i in 0..self.definitions.len() {
            definitions.push_str(&self.definitions[i].to_string());
        }

        write!(f, "\n{}{}", self.part_of_speech, definitions)
    }
}

#[derive(Deserialize, Debug)]
struct Definition {
    definition: String,
    synonyms: Option<Vec<String>>,
    example: Option<String>,
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\t• {}", self.definition)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead};
    use std::fs::{self, File};

    use super::*;

    fn read_from_file(path: &str) -> io::Lines<io::BufReader<File>> {
        let f = File::open(path)
                    .expect(format!("Error reading {}", path).as_str());
       io::BufReader::new(f).lines()
    }

    fn get_json(code: &str) {
        use fs::OpenOptions;
        use io::Write;

        let words = read_from_file(format!("./tests/{}/{}_words.txt", code, code).as_str());
        File::create(format!("./tests/{}/{}_json.txt", code, code).as_str())
            .expect(format!("Could not create {}_json.txt", code).as_str());
        let mut file = OpenOptions::new()
            .append(true)
            .open(format!("./tests/{}/{}_json.txt", code, code).as_str())
            .unwrap();
        
        for word in words {
            let word = word.unwrap();
            println!("Downloading: {}", word);
            let response = tokio_test::block_on(reqwest::get(
                format!("https://api.dictionaryapi.dev/api/v2/entries/{}/{}",
                code, word
            ))).unwrap();
            let json_string = tokio_test::block_on(response.text()).unwrap();
            writeln!(file, "{}", json_string)
                .expect(format!("Failed to write {} result to file", word).as_str());
        }
    }
    
    // en_US
    // Tests if succesful responses are serialized correctly
    #[test]
    fn en_us_ser_tests() {
        if !std::path::Path::new("./tests/en_US/en_US_json.txt").exists() {
            println!("Downloading words...");
            get_json("en_US");
        }

        let lines = read_from_file("./tests/en_US/en_US_json.txt");

        for line in lines {
            let word: Vec<Word> = serde_json::from_str(line.unwrap().as_str()).unwrap();
            println!("Testing: {}", word[0].word);
        }
    }

    // es
    // Tests if succesful responses are serialized correctly
    #[test]
    fn es_ser_tests() {
        if !std::path::Path::new("./tests/es/es_json.txt").exists() {
            println!("Downloading words...");
            get_json("es");
        }

        let lines = read_from_file("./tests/es/es_json.txt");

        for line in lines {
            let word: Vec<Word> = serde_json::from_str(line.unwrap().as_str()).unwrap();
            println!("Testing: {}", word[0].word);
        }
    }
}
