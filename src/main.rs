use serde::Deserialize;
use std::env;
use std::fmt;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let thing = get_word(String::from(&args[1])).await;
    match thing {
        Ok(def) => {
            print!("{}", &def[0]);
            Ok(())
        }
        Err(e) => {
            println!("Error: Word not found. {}", e);
            Err(e)
        },
    }
}

async fn get_word(word: String) -> Result<Vec<Word>, reqwest::Error> {
    let response: Vec<Word> = reqwest::get(format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en_US/{}",
        word
    ))
    .await?
    .json()
    .await?;
    Ok(response)
}

#[derive(Deserialize, Debug)]
struct Word {
    word: String,
    phonetics: Vec<Phonetic>,
    meanings: Vec<Meaning>,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut phonetics: String = String::new();

        phonetics.push_str(&self.phonetics[0].to_string());

        for i in 1..self.phonetics.len() {
            phonetics.push_str(", ");
            phonetics.push_str(&self.phonetics[i].to_string());
        }

        let mut meanings: String = String::new();

        meanings.push_str(&self.meanings[0].to_string());

        for i in 1..self.meanings.len() {
            meanings.push_str(&self.meanings[i].to_string());
        }

        write!(f, "{} [{}]{}", self.word, phonetics, meanings)
    }
}

#[derive(Deserialize, Debug)]
struct Phonetic {
    text: String,
    audio: String,
}

impl fmt::Display for Phonetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
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

    #[test]
    fn top1000_test() {
        use std::fs;
        use rand::Rng;
        
        let words = fs::read_to_string("./top1000.txt").expect("Error reading top1000.txt");
        let words = words_by_line(&words);
        
        let mut rng = rand::thread_rng();
        for _ in 1..5 {
            let temp = rng.gen_range(0..999);
            tokio_test::block_on(get_word(words[temp][0].to_owned())).unwrap();
        }
    }
}