use serde_json::{self, Map};

fn decode_bencoded_value_rec(encoded_value: &str) -> (serde_json::Value, &str) {
    let c = encoded_value.chars().next().expect("Found an empty string");

    match c {
        '0'..='9' => {
            let (letters_count, rest) = encoded_value
                .split_once(':')
                .expect("Expected an ':' but found none");

            let number = letters_count.parse::<i64>().unwrap() as usize;
            let string = &rest[..number];
            (
                serde_json::Value::String(string.to_string()),
                &rest[number..],
            )
        }
        'l' => {
            let mut v: Vec<serde_json::Value> = Vec::new();
            let mut str = &encoded_value[1..];
            while str.chars().next().unwrap() != 'e' {
                let (val, remaining) = decode_bencoded_value_rec(str);

                v.push(val);
                str = remaining;
            }

            (serde_json::Value::Array(v), &str[1..])
        }
        'd' => {
            let mut map: Map<String, serde_json::Value> = Map::new();
            let mut str = &encoded_value[1..];
            while str.chars().next().unwrap() != 'e' {
                let (key, remaining) = decode_bencoded_value_rec(str);
                let (v, remaining) = decode_bencoded_value_rec(remaining);

                let k = key
                    .as_str()
                    .expect(&format!(
                        "Expected the key to be a string but found {:?}",
                        key
                    ))
                    .to_owned();
                map.insert(k, v);
                str = remaining;
            }

            (serde_json::Value::Object(map), &str[1..])
        }
        'i' => {
            let (str_num, rest) = encoded_value[1..]
                .split_once('e')
                .expect("Expected an 'e' at the end of the number but found none");
            let num = str_num.parse::<i64>().unwrap();
            (serde_json::Value::Number(num.into()), rest)
        }
        _ => {
            panic!("Unknown encoding: {}", c)
        }
    }
}

pub fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let result = decode_bencoded_value_rec(encoded_value);
    eprintln!("result: {}, '{}'", result.0, result.1);
    return decode_bencoded_value_rec(encoded_value).0;
}
