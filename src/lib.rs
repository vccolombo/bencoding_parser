pub mod bencoding_parser {
    use std::collections::HashMap;

    #[cfg(debug_assertions)]
    macro_rules! debug {
        ($x:expr) => {
            dbg!($x)
        };
    }

    #[cfg(not(debug_assertions))]
    macro_rules! debug {
        ($x:expr) => {
            std::convert::identity($x)
        };
    }

    #[derive(Debug)]
    pub enum BencodingError {}

    #[derive(Debug, Clone)]
    pub enum BencodingValue {
        String(String),
        Dict(HashMap<String, BencodingValue>),
    }

    pub struct Bencoding {
        dict: HashMap<String, BencodingValue>,
    }

    impl Bencoding {
        pub fn decode(data: &[u8]) -> Result<Self, BencodingError> {
            let (dict, _) = Self::decode_dict(data);

            debug!(&dict);

            return Ok(Self { dict });
        }

        pub fn get(&self, key: &str) -> Option<BencodingValue> {
            if !self.dict.contains_key(key) {
                return None;
            }

            return Some(self.dict[key].clone());
        }

        fn decode_dict(mut data: &[u8]) -> (HashMap<String, BencodingValue>, &[u8]) {
            data = &data[1..];
            let mut key;
            let mut value;

            let mut dict = HashMap::new();
            loop {
                // 0x65 ('e') indicates end of dictionary
                if data[0] == 'e' as u8 {
                    break;
                }

                (key, data) = Self::decode_string(data);
                (value, data) = Self::decode_next(data);
                dict.insert(key, value);
            }

            return (dict, data);
        }

        fn decode_string(mut data: &[u8]) -> (String, &[u8]) {
            let mut separator_idx = 0;

            while data[separator_idx] != ':' as u8 {
                separator_idx = separator_idx + 1;
            }

            let length = std::str::from_utf8(&data[..separator_idx])
                .unwrap()
                .parse()
                .unwrap();
            data = &data[separator_idx + 1..];
            let value = std::str::from_utf8(&data[..length]).unwrap();
            data = &data[length..];

            return (value.to_owned(), data);
        }

        fn decode_next(data: &[u8]) -> (BencodingValue, &[u8]) {
            match data[0] as char {
                'i' => todo!(), // integer
                'l' => todo!(), // list
                'd' => {
                    let (value, data) = Self::decode_dict(&data);
                    return (BencodingValue::Dict(value), data);
                }
                _ => {
                    let (value, data) = Self::decode_string(&data);
                    return (BencodingValue::String(value), data);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bencoding_parser::{Bencoding, BencodingValue};

    #[test]
    fn decode_string_key_hello_value_world() {
        let parser = Bencoding::decode(b"d5:hello5:worlde").unwrap();
        let result = match parser.get("hello").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "world");
    }

    #[test]
    fn decode_string_key_key_value_value() {
        let parser = Bencoding::decode(b"d3:key5:valuee").unwrap();
        let result = match parser.get("key").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "value");
    }

    #[test]
    fn decode_string_utf8() {
        let parser = Bencoding::decode("d6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get("author").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "Víctor Colombo");
    }

    #[test]
    fn decode_multiple_strings_first() {
        let parser =
            Bencoding::decode("d3:key5:value6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get("key").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "value");
    }

    #[test]
    fn decode_multiple_strings_second() {
        let parser =
            Bencoding::decode("d3:key5:value6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get("author").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "Víctor Colombo");
    }

    #[test]
    fn decode_dict_inside_dict() {
        let parser = Bencoding::decode(
            "d6:author15:Víctor Colombo16:dict_inside_dictd3:key5:valueee".as_bytes(),
        )
        .unwrap();
        let dict = match parser.get("dict_inside_dict").unwrap() {
            BencodingValue::Dict(d) => d,
            _ => panic!(),
        };
        let result = match &dict["key"] {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "value");
    }

    #[test]
    fn decode_empty_dict_does_not_panic() {
        Bencoding::decode(b"de").unwrap();
    }

    #[test]
    fn get_key_that_does_not_exist_must_return_none() {
        let parser = Bencoding::decode(b"de").unwrap();
        let result = parser.get("fake");
        assert!(result.is_none());
    }
}
