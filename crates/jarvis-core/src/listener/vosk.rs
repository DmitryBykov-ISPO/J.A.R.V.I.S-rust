use crate::{config, stt, i18n};

pub fn init() -> Result<(), ()> {
    Ok(()) // nothing to init for Vosk
}

pub fn data_callback(frame_buffer: &[i16]) -> Option<i32> {
    if let Some((recognized, _confidence)) = stt::recognize_wake_word(frame_buffer) {
        let recognized = recognized.trim().to_lowercase();
        
        // skip unknown/empty
        if recognized.is_empty() || recognized == "[unk]" {
            return None;
        }
        
        info!("Wake word candidate: '{}'", recognized);
        
        // language-specific wake phrase
        let lang = i18n::get_language();
        let wake_phrase = config::get_wake_phrase(&lang);

        // verify with seqdiff ratio
        let wake_chars: Vec<char> = wake_phrase.chars().collect();
        let recognized_chars: Vec<char> = recognized.chars().collect();
        let similarity = seqdiff::ratio(&wake_chars, &recognized_chars);
        
        info!("Similarity: {:.1}% ('{}' vs '{}')", similarity, recognized, config::VOSK_FETCH_PHRASE);
        
        if similarity >= config::VOSK_MIN_RATIO {
            info!("Wake word activated!");
            return Some(0);
        }
    }
    
    None
}

// @TODO. Make it better somehow (more accurate or with higher sensitivity).
// pub fn data_callback(frame_buffer: &[i16]) -> Option<i32> {
//     // recognize & convert to sequence
//     let recognized_phrase = stt::recognize(&frame_buffer, true).unwrap_or("".into());

//     if !recognized_phrase.trim().is_empty() {
//         info!("Vosk wake-word debug info:");
//         info!("rec: {}", recognized_phrase);
//         let recognized_phrases = recognized_phrase.split_whitespace();
//         for phrase in recognized_phrases {
//             let recognized_phrase_chars = phrase.trim().to_lowercase().chars().collect::<Vec<_>>();

//             // compare
//             let compare_ratio = seqdiff::ratio(
//                 &config::VOSK_FETCH_PHRASE.chars().collect::<Vec<_>>(),
//                 &recognized_phrase_chars,
//             );
//             info!("og phrase: {:?}", &config::VOSK_FETCH_PHRASE);
//             info!("recognized phrase: {:?}", &recognized_phrase_chars);
//             info!("compare ratio: {}", compare_ratio);

//             if compare_ratio >= config::VOSK_MIN_RATIO {
//                 info!("Phrase activated.");
//                 return Some(0);
//             }
//         }
//     }

//     None
// }
