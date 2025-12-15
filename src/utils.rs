use chrono::DateTime;
use xml::ParserConfig;


pub fn clean_string(s: &str) -> String {
    s.trim().replace(r#"(\r\n|\n|\r)"#, "")
}

pub fn truncate_string(s: &str, length: usize) -> String {
    s.chars().take(length).collect()
}

pub fn truncate_int(number: i32) -> i32 {
    number.clamp(-2147483647, 2147483647)
}

fn contains_non_latin_codepoints(s: &str) -> bool {
    s.chars().any(|c| c as u32 > 0x00FF)
}

fn replace_non_latin_characters(s: &str) -> String {
    s.replace(r#"[^\x00-\x80]"#, " ")
}

pub fn sanitize_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }

    if contains_non_latin_codepoints(url) {
        let encoded = urlencoding::encode(&url);
        let mut new_url = truncate_string(&encoded, 768);

        if contains_non_latin_codepoints(&new_url) {
            new_url = replace_non_latin_characters(&new_url);
        }

        return truncate_string(&new_url, 768);
    }

    truncate_string(url, 768)
}

pub fn pub_date_to_timestamp(pub_date: &str) -> i64 {
    let pub_date_str = pub_date.trim();
    if pub_date_str.is_empty() {
        return 0; // bad pub date, return 0
    }

    if let Ok(num) = pub_date_str.parse::<i64>() {
        return num; // already a timestamp
    }

    // parse rfc 2882 (rss spec) and iso 8601 (rfc 3339)
    DateTime::parse_from_rfc2822(pub_date_str)
        .or_else(|_| DateTime::parse_from_rfc3339(pub_date_str))
        .map(|dt| dt.timestamp())
        .unwrap_or(0) // return timestamp or 0 if error
}



pub fn calculate_update_frequency(pubdates: &[i64]) -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let time_400_days_ago = now - (400 * 24 * 60 * 60);
    let time_200_days_ago = now - (200 * 24 * 60 * 60);
    let time_100_days_ago = now - (100 * 24 * 60 * 60);
    let time_40_days_ago = now - (40 * 24 * 60 * 60);
    let time_20_days_ago = now - (20 * 24 * 60 * 60);
    let time_10_days_ago = now - (10 * 24 * 60 * 60);
    let time_5_days_ago = now - (5 * 24 * 60 * 60);

    if pubdates.iter().filter(|&&time| time > time_400_days_ago).count() == 0 {
        return 9;
    }
    if pubdates.iter().filter(|&&time| time > time_200_days_ago).count() == 0 {
        return 8;
    }
    if pubdates.iter().filter(|&&time| time > time_100_days_ago).count() == 0 {
        return 7;
    }
    if pubdates.iter().filter(|&&time| time > time_5_days_ago).count() > 1 {
        return 1;
    }
    if pubdates.iter().filter(|&&time| time > time_10_days_ago).count() > 1 {
        return 2;
    }
    if pubdates.iter().filter(|&&time| time > time_20_days_ago).count() > 1 {
        return 3;
    }
    if pubdates.iter().filter(|&&time| time > time_40_days_ago).count() > 1 {
        return 4;
    }
    if pubdates.iter().filter(|&&time| time > time_100_days_ago).count() > 1 {
        return 5;
    }
    if pubdates.iter().filter(|&&time| time > time_200_days_ago).count() > 1 {
        return 6;
    }
    if pubdates.iter().filter(|&&time| time > time_400_days_ago).count() >= 1 {
        return 7;
    }
    0
}

//Get a mime-type string for an unknown media enclosure
pub fn guess_enclosure_type(url: &str) -> String {
    if url.contains(".m4v") {
        return "video/mp4".to_string();
    }
    if url.contains(".mp4") {
        return "video/mp4".to_string();
    }
    if url.contains(".avi") {
        return "video/avi".to_string();
    }
    if url.contains(".mov") {
        return "video/quicktime".to_string();
    }
    if url.contains(".mp3") {
        return "audio/mpeg".to_string();
    }
    if url.contains(".m4a") {
        return "audio/mp4".to_string();
    }
    if url.contains(".wav") {
        return "audio/wav".to_string();
    }
    if url.contains(".ogg") {
        return "audio/ogg".to_string();
    }
    if url.contains(".wmv") {
        return "video/x-ms-wmv".to_string();
    }

    "".to_string()
}

/*
* Convert time string to seconds
* 01:02 = 62 seconds
* Thanks to Glenn Bennett!
*/
pub fn time_to_seconds(time_string: &str) -> i32 {
    let parts = time_string.split(':').collect::<Vec<&str>>();

    match parts.len() - 1 {
        1 => {
            let minutes = parts[0].parse::<i32>().unwrap_or(0);
            let seconds = parts[1].parse::<i32>().unwrap_or(0);
            minutes * 60 + seconds
        }
        2 => {
            let hours = parts[0].parse::<i32>().unwrap_or(0);
            let minutes = parts[1].parse::<i32>().unwrap_or(0);
            let seconds = parts[2].parse::<i32>().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        _ => time_string.parse::<i32>().unwrap_or(30 * 60),
    }
}

pub fn add_html_entities_to_parser_config(config: ParserConfig) -> ParserConfig {
    config
        .add_entity("amp", "&")
        .add_entity("lt", "<")
        .add_entity("gt", ">")
        .add_entity("nbsp", " ")
        .add_entity("iexcl", "¡")
        .add_entity("cent", "¢")
        .add_entity("pound", "£")
        .add_entity("curren", "¤")
        .add_entity("yen", "¥")
        .add_entity("brvbar", "¦")
        .add_entity("sect", "§")
        .add_entity("uml", "¨")
        .add_entity("copy", "©")
        .add_entity("ordf", "ª")
        .add_entity("laquo", "«")
        .add_entity("not", "¬")
        .add_entity("shy", "­")
        .add_entity("reg", "®")
        .add_entity("macr", "¯")
        .add_entity("deg", "°")
        .add_entity("plusmn", "±")
        .add_entity("sup2", "²")
        .add_entity("sup3", "³")
        .add_entity("acute", "´")
        .add_entity("micro", "µ")
        .add_entity("para", "¶")
        .add_entity("cedil", "¸")
        .add_entity("sup1", "¹")
        .add_entity("ordm", "º")
        .add_entity("raquo", "»")
        .add_entity("frac14", "¼")
        .add_entity("frac12", "½")
        .add_entity("frac34", "¾")
        .add_entity("iquest", "¿")
        .add_entity("times", "×")
        .add_entity("divide", "÷")
        .add_entity("forall", "∀")
        .add_entity("part", "∂")
        .add_entity("exist", "∃")
        .add_entity("empty", "∅")
        .add_entity("nabla", "∇")
        .add_entity("isin", "∈")
        .add_entity("notin", "∉")
        .add_entity("ni", "∋")
        .add_entity("prod", "∏")
        .add_entity("sum", "∑")
        .add_entity("minus", "−")
        .add_entity("lowast", "∗")
        .add_entity("radic", "√")
        .add_entity("prop", "∝")
        .add_entity("infin", "∞")
        .add_entity("ang", "∠")
        .add_entity("and", "∧")
        .add_entity("or", "∨")
        .add_entity("cap", "∩")
        .add_entity("cup", "∪")
        .add_entity("int", "∫")
        .add_entity("there4", "∴")
        .add_entity("sim", "∼")
        .add_entity("cong", "≅")
        .add_entity("asymp", "≈")
        .add_entity("ne", "≠")
        .add_entity("equiv", "≡")
        .add_entity("le", "≤")
        .add_entity("ge", "≥")
        .add_entity("sub", "⊂")
        .add_entity("sup", "⊃")
        .add_entity("nsub", "⊄")
        .add_entity("sube", "⊆")
        .add_entity("supe", "⊇")
        .add_entity("oplus", "⊕")
        .add_entity("otimes", "⊗")
        .add_entity("perp", "⊥")
        .add_entity("sdot", "⋅")
        .add_entity("Alpha", "Α")
        .add_entity("Beta", "Β")
        .add_entity("Gamma", "Γ")
        .add_entity("Delta", "Δ")
        .add_entity("Epsilon", "Ε")
        .add_entity("Zeta", "Ζ")
        .add_entity("Eta", "Η")
        .add_entity("Theta", "Θ")
        .add_entity("Iota", "Ι")
        .add_entity("Kappa", "Κ")
        .add_entity("Lambda", "Λ")
        .add_entity("Mu", "Μ")
        .add_entity("Nu", "Ν")
        .add_entity("Xi", "Ξ")
        .add_entity("Omicron", "Ο")
        .add_entity("Pi", "Π")
        .add_entity("Rho", "Ρ")
        .add_entity("Sigma", "Σ")
        .add_entity("Tau", "Τ")
        .add_entity("Upsilon", "Υ")
        .add_entity("Phi", "Φ")
        .add_entity("Chi", "Χ")
        .add_entity("Psi", "Ψ")
        .add_entity("Omega", "Ω")
        .add_entity("alpha", "α")
        .add_entity("beta", "β")
        .add_entity("gamma", "γ")
        .add_entity("delta", "δ")
        .add_entity("epsilon", "ε")
        .add_entity("zeta", "ζ")
        .add_entity("eta", "η")
        .add_entity("theta", "θ")
        .add_entity("iota", "ι")
        .add_entity("kappa", "κ")
        .add_entity("lambda", "λ")
        .add_entity("mu", "μ")
        .add_entity("nu", "ν")
        .add_entity("xi", "ξ")
        .add_entity("omicron", "ο")
        .add_entity("pi", "π")
        .add_entity("rho", "ρ")
        .add_entity("sigmaf", "ς")
        .add_entity("sigma", "σ")
        .add_entity("tau", "τ")
        .add_entity("upsilon", "υ")
        .add_entity("phi", "φ")
        .add_entity("chi", "χ")
        .add_entity("psi", "ψ")
        .add_entity("omega", "ω")
        .add_entity("thetasym", "ϑ")
        .add_entity("upsih", "ϒ")
        .add_entity("piv", "ϖ")
        .add_entity("OElig", "Œ")
        .add_entity("oelig", "œ")
        .add_entity("Scaron", "Š")
        .add_entity("scaron", "š")
        .add_entity("Yuml", "Ÿ")
        .add_entity("fnof", "ƒ")
        .add_entity("circ", "ˆ")
        .add_entity("tilde", "˜")
        .add_entity("ensp", " ")
        .add_entity("emsp", " ")
        .add_entity("thinsp", " ")
        .add_entity("zwnj", "\u{200c}")
        .add_entity("zwj", "\u{200d}")
        .add_entity("lrm", "\u{200e}")
        .add_entity("rlm", "\u{200f}")
        .add_entity("ndash", "–")
        .add_entity("mdash", "—")
        .add_entity("lsquo", "‘")
        .add_entity("rsquo", "’")
        .add_entity("sbquo", "‚")
        .add_entity("ldquo", "“")
        .add_entity("rdquo", "”")
        .add_entity("bdquo", "„")
        .add_entity("dagger", "†")
        .add_entity("Dagger", "‡")
        .add_entity("bull", "•")
        .add_entity("hellip", "…")
        .add_entity("permil", "‰")
        .add_entity("prime", "′")
        .add_entity("Prime", "″")
        .add_entity("lsaquo", "‹")
        .add_entity("rsaquo", "›")
        .add_entity("oline", "‾")
        .add_entity("euro", "€")
        .add_entity("trade", "™")
        .add_entity("larr", "←")
        .add_entity("uarr", "↑")
        .add_entity("rarr", "→")
        .add_entity("darr", "↓")
        .add_entity("harr", "↔")
        .add_entity("crarr", "↵")
        .add_entity("lceil", "⌈")
        .add_entity("rceil", "⌉")
        .add_entity("lfloor", "⌊")
        .add_entity("rfloor", "⌋")
        .add_entity("loz", "◊")
        .add_entity("spades", "♠")
        .add_entity("clubs", "♣")
        .add_entity("hearts", "♥")
        .add_entity("diams", "♦")
}