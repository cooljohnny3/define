use serde::Deserialize;
use std::env;
use std::fmt;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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
                        .help("Sets the language to search in. default=en_US (Not yet supported)"))
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
            println!("Error: Word not found.");
            Err(e)
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

    #[test]
    fn word_test1() {
        let case1: Word = serde_json::from_str(
            r#"  {
            "word": "hello",
            "phonetics": [
              {
                "text": "/həˈloʊ/",
                "audio": "https://lex-audio.useremarkable.com/mp3/hello_us_1_rr.mp3"
              },
              {
                "text": "/hɛˈloʊ/",
                "audio": "https://lex-audio.useremarkable.com/mp3/hello_us_2_rr.mp3"
              }
            ],
            "meanings": [
              {
                "partOfSpeech": "noun",
                "definitions": [
                  {
                    "definition": "An utterance of “hello”; a greeting.",
                    "synonyms": [
                      "greeting",
                      "welcome",
                      "salutation",
                      "saluting",
                      "hailing",
                      "address",
                      "hello",
                      "hallo"
                    ],
                    "example": "she was getting polite nods and hellos from people"
                  }
                ]
              },
              {
                "partOfSpeech": "intransitive verb",
                "definitions": [
                  {
                    "definition": "Say or shout “hello”; greet someone.",
                    "example": "I pressed the phone button and helloed"
                  }
                ]
              },
              {
                "partOfSpeech": "exclamation",
                "definitions": [
                  {
                    "definition": "Used as a greeting or to begin a phone conversation.",
                    "example": "hello there, Katie!"
                  }
                ]
              }
            ]
          }"#,
        )
        .unwrap();
        assert_eq!(
            case1.to_string(),
            "hello [/həˈloʊ/, /hɛˈloʊ/]\
        \nnoun\
            \n\t• An utterance of “hello”; a greeting.\
        \nintransitive verb\
            \n\t• Say or shout “hello”; greet someone.\
        \nexclamation\
            \n\t• Used as a greeting or to begin a phone conversation."
        )
    }

    #[test]
    fn word_test2() {
        let case1: Word = serde_json::from_str(
            r#"{"word":"dog","phonetics":[{"text":"/dɔɡ/","audio":"https://lex-audio.useremarkable.com/mp3/dog_us_1_rr.mp3"}],"meanings":[{"partOfSpeech":"transitive verb","definitions":[{"definition":"Follow (someone or their movements) closely and persistently.","synonyms":["pursue","follow","stalk","track","trail","shadow","hound"],"example":"photographers seemed to dog her every step"},{"definition":"Act lazily; fail to try one's hardest."},{"definition":"Grip (something) with a mechanical device.","example":"she has dogged the door shut"}]},{"partOfSpeech":"noun","definitions":[{"definition":"A domesticated carnivorous mammal that typically has a long snout, an acute sense of smell, nonretractable claws, and a barking, howling, or whining voice.","synonyms":["canine","hound"],"example":"‘All dogs have an intense sense of smell, and every dog likes to sniff,’ Smith said."},{"definition":"An unpleasant, contemptible, or wicked man."},{"definition":"A mechanical device for gripping."},{"definition":"Feet.","synonyms":["tootsie","trotter"]},{"definition":"short for firedog"}]}]}"#,
        )
        .unwrap();
        assert_eq!(
            case1.to_string(),
            "dog [/dɔɡ/]\
            \ntransitive verb\
                \n\t• Follow (someone or their movements) closely and persistently.\
                \n\t• Act lazily; fail to try one's hardest.\
                \n\t• Grip (something) with a mechanical device.\
            \nnoun\
                \n\t• A domesticated carnivorous mammal that typically has a long snout, an acute sense of smell, nonretractable claws, and a barking, howling, or whining voice.\
                \n\t• An unpleasant, contemptible, or wicked man.\
                \n\t• A mechanical device for gripping.\
                \n\t• Feet.\
                \n\t• short for firedog"
        )
    }

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
    #[test]
    fn es_tests() {
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
