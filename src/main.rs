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

    fn words_by_line<'a>(s: &'a str) -> Vec<Vec<&'a str>> {
        s.lines().map(|line| {
            line.split_whitespace().collect()
        }).collect()
    }

    fn get_en_us_json() {
        use std::fs;
        
        let words = fs::read_to_string("./tests/en_US_words.txt")
            .expect("Error reading en_US_words.txt");
        let words = words_by_line(&words);
        fs::File::create("./tests/en_US/en_US_json.txt")
            .expect("Error creating file: en_US_json.txt");
        
        let mut contents = String::new();
        for i in 0..100 {
            let word = words[i][0];
            println!("{}", word);
            let response = tokio_test::block_on(reqwest::get(
                format!("https://api.dictionaryapi.dev/api/v2/entries/en_US/{}",
                word
            ))).unwrap();
            let json_string = tokio_test::block_on(response.text()).unwrap();
            contents.push_str(&json_string);
            contents.push_str("\n");
        }
        fs::write("./tests/en_US/en_US_json.txt", contents).expect("failed to write");
    }

    #[test]
    fn en_us_ser_tests() {
        use std::fs;

        if !std::path::Path::new("./tests/en_US/en_US_json.txt").exists() {
            get_en_us_json();
        }
        
        let cases = fs::read_to_string(
            "./tests/en_US/en_US_json.txt")
            .expect("Error reading en_US_json.txt");
        let cases = words_by_line(&cases);
        
        for i in 0..cases.len() {
            println!("{}", cases[i][0]);
            let _: Vec<Word> = serde_json::from_str(cases[i][0]).unwrap();
        }
    }

    // fn get_es_tests() {
    //     use std::fs;
    //     use rand::Rng;
        
    //     let words = fs::read_to_string("./top1000.txt").expect("Error reading top1000.txt");
    //     let words = words_by_line(&words);
        
    //     let mut rng = rand::thread_rng();
    //     for _ in 1..5 {
    //         let temp = rng.gen_range(0..999);
    //         tokio_test::block_on(get_word(words[temp][0].to_owned(), String::from("en_US"))).unwrap();
    //     }
    // }

    // #[test]
    // fn es_tests() {
    //     use std::fs;
    //     use rand::Rng;
        
    //     let words = fs::read_to_string("./top1000.txt").expect("Error reading top1000.txt");
    //     let words = words_by_line(&words);
        
    //     let mut rng = rand::thread_rng();
    //     for _ in 1..5 {
    //         let temp = rng.gen_range(0..999);
    //         tokio_test::block_on(get_word(words[temp][0].to_owned(), String::from("en_US"))).unwrap();
    //     }
    // }
}
