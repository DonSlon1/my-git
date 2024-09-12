use std::hash::RandomState;
use ordermap::OrderMap;
use sha1::digest::typenum::Integer;

pub fn kvlm_parse<'a>(raw: &'a [u8], start: Option<usize>, dct: Option<OrderMap<Vec<u8>,Vec<Vec<u8>>,RandomState>>) -> OrderMap<Vec<u8>,Vec<Vec<u8>>,RandomState> {
    let mut dct = dct.unwrap_or(OrderMap::new());
    let start = start.unwrap_or(0);

    let spc = raw.iter().skip(start).position(|&v| v == b' ').map(|pos| pos + start);
    let nl = raw.iter().skip(start).position(|&v| v == b'\n').map(|pos| pos + start);
    
    if spc.is_none() || nl < spc {
        dct.insert(b"None".to_vec(), vec!(Vec::from(raw.split_at(start + 1).1)));
        return dct
    }
    
    let spc = spc.unwrap();
    let key = &raw[start..spc];

    let mut end = start;
    loop {
        match raw.iter().skip(end + 1).position(|&v| v == b'\n').map(|pos| pos + end + 1) {
            None => break,
            Some(v) => {
                end = v;
            }
        }
        // Check if the next character exists and is not a space
        if end + 1 >= raw.len() || raw[end + 1] != b' ' {
            break;
        }
    }

    let slice:&[u8] = &raw[spc+1..end];
    let data = slice.clone()
        .chunks(2)
        .flat_map(|chunk| {
            if chunk == b"\n ".as_ref() {
                b"\n".iter().copied()
            } else {
                chunk.iter().copied()
            }
        }).collect::<Vec<u8>>().to_owned();

    if let Some(existing) = dct.get_mut(key) {
        existing.push(data);
    } else {
        dct.insert(key.to_vec(), vec![data]);
    }

    kvlm_parse(raw,Some(end+1),Some(dct))
}


pub fn kvlm_serialize(kvlm: &OrderMap<Vec<u8>, Vec<Vec<u8>>, RandomState>) -> String {
    let mut ret = String::new();

    for (key, values) in kvlm.iter() {
        if key == b"None" { continue }
        // Convert key to a String
        if let Ok(key_str) = String::from_utf8(key.clone()) {
            for value in values {
                // Convert value to a String
                if let Ok(value_str) = String::from_utf8(value.clone()) {
                    // Replace '\n' with '\n '
                    let replaced_value = value_str.replace('\n', "\n ");

                    // Append key, replaced value, and newline
                    ret.push_str(&key_str);
                    ret.push(' ');
                    ret.push_str(&replaced_value);
                    ret.push('\n');
                }
            }
        }
    }

    // Append message
    if let Some(message) = kvlm.get(&b"None".to_vec()) {
        ret.push('\n');
        for line in message {
            if let Ok(line_str) = String::from_utf8(line.clone()) {
                ret.push_str(&line_str);
            }
        }
        ret.push('\n');
    }

    ret
}
