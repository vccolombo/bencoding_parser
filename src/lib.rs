pub mod bencoding_parser {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum BencodingError {}

    #[derive(Debug, Clone)]
    pub enum BencodingValue {
        String(Vec<u8>),
        Integer(i64),
        Dict(HashMap<Vec<u8>, BencodingValue>),
        List(Vec<BencodingValue>),
    }

    pub struct Bencoding {
        dict: HashMap<Vec<u8>, BencodingValue>,
    }

    impl Bencoding {
        pub fn decode(data: &[u8]) -> Result<Self, BencodingError> {
            let (dict, _) = Self::decode_dict(data);

            return Ok(Self { dict });
        }

        pub fn get(&self, key: &[u8]) -> Option<BencodingValue> {
            if !self.dict.contains_key(key) {
                return None;
            }

            return Some(self.dict[key].clone());
        }

        fn decode_dict(mut data: &[u8]) -> (HashMap<Vec<u8>, BencodingValue>, &[u8]) {
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

        fn decode_string(mut data: &[u8]) -> (Vec<u8>, &[u8]) {
            let mut separator_idx = 0;

            while data[separator_idx] != ':' as u8 {
                separator_idx = separator_idx + 1;
            }

            let length = std::str::from_utf8(&data[..separator_idx])
                .unwrap()
                .parse()
                .unwrap();
            data = &data[separator_idx + 1..];
            let value = data[..length].to_vec();
            data = &data[length..];

            return (value, data);
        }

        fn decode_integer(mut data: &[u8]) -> (i64, &[u8]) {
            // TODO: i-0e is invalid. All encodings with a leading zero, such as i03e, are
            // invalid, other than i0e, which of course corresponds to the integer "0".
            data = &data[1..];
            let mut ending_idx = 0;
            while data[ending_idx] != 'e' as u8 {
                ending_idx = ending_idx + 1;
            }

            let value = std::str::from_utf8(&data[..ending_idx])
                .unwrap()
                .parse()
                .unwrap();

            return (value, &data[ending_idx + 1..]);
        }

        fn decode_list(mut data: &[u8]) -> (Vec<BencodingValue>, &[u8]) {
            data = &data[1..];
            let mut value;

            let mut list: Vec<BencodingValue> = Vec::new();
            loop {
                // 0x65 ('e') indicates end of dictionary
                if data[0] == 'e' as u8 {
                    break;
                }

                (value, data) = Self::decode_next(data);
                list.push(value);
            }

            return (list, data);
        }

        fn decode_next(data: &[u8]) -> (BencodingValue, &[u8]) {
            match data[0] as char {
                'i' => {
                    let (value, data) = Self::decode_integer(&data);
                    return (BencodingValue::Integer(value), data);
                }
                'l' => {
                    let (value, data) = Self::decode_list(&data);
                    return (BencodingValue::List(value), data);
                }
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
        let result = match parser.get(b"hello").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, b"world");
    }

    #[test]
    fn decode_string_key_key_value_value() {
        let parser = Bencoding::decode(b"d3:key5:valuee").unwrap();
        let result = match parser.get(b"key").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, b"value");
    }

    #[test]
    fn decode_string_utf8() {
        let parser = Bencoding::decode("d6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get(b"author").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "Víctor Colombo".as_bytes());
    }

    #[test]
    fn decode_multiple_strings_first() {
        let parser =
            Bencoding::decode("d3:key5:value6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get(b"key").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, b"value");
    }

    #[test]
    fn decode_multiple_strings_second() {
        let parser =
            Bencoding::decode("d3:key5:value6:author15:Víctor Colomboe".as_bytes()).unwrap();
        let result = match parser.get(b"author").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, "Víctor Colombo".as_bytes());
    }

    #[test]
    fn decode_non_utf8_string() {
        let parser = Bencoding::decode(b"d3:key5:\xAB\xA3\xDA\x89\xFCe").unwrap();
        let result = match parser.get(b"key").unwrap() {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, b"\xAB\xA3\xDA\x89\xFC");
    }

    #[test]
    fn decode_one_digit_integer_5() {
        let parser = Bencoding::decode("d7:integeri5ee".as_bytes()).unwrap();
        let result = match parser.get(b"integer").unwrap() {
            BencodingValue::Integer(i) => i,
            _ => panic!(),
        };
        assert_eq!(result, 5);
    }

    #[test]
    fn decode_one_digit_integer_6() {
        let parser = Bencoding::decode("d7:integeri6ee".as_bytes()).unwrap();
        let result = match parser.get(b"integer").unwrap() {
            BencodingValue::Integer(i) => i,
            _ => panic!(),
        };
        assert_eq!(result, 6);
    }

    #[test]
    fn decode_two_digits_integer_42() {
        let parser = Bencoding::decode("d7:integeri42ee".as_bytes()).unwrap();
        let result = match parser.get(b"integer").unwrap() {
            BencodingValue::Integer(i) => i,
            _ => panic!(),
        };
        assert_eq!(result, 42);
    }

    #[test]
    fn decode_three_digits_negative_integer_minus_18() {
        let parser = Bencoding::decode("d7:integeri-18ee".as_bytes()).unwrap();
        let result = match parser.get(b"integer").unwrap() {
            BencodingValue::Integer(i) => i,
            _ => panic!(),
        };
        assert_eq!(result, -18);
    }

    #[test]
    fn decode_dict_inside_dict() {
        let parser = Bencoding::decode(
            "d6:author15:Víctor Colombo16:dict_inside_dictd3:key5:valueee".as_bytes(),
        )
        .unwrap();
        let dict = match parser.get(b"dict_inside_dict").unwrap() {
            BencodingValue::Dict(d) => d,
            _ => panic!(),
        };
        let result = match &dict[&b"key".to_vec()] {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        assert_eq!(result, b"value");
    }

    #[test]
    fn decode_list_with_two_elements() {
        let parser = Bencoding::decode("d4:listl5:elem1i42eee".as_bytes()).unwrap();
        let bv = parser.get(b"list").unwrap();
        let list = match bv {
            BencodingValue::List(l) => l,
            _ => panic!(),
        };
        let elem1 = match &list[0] {
            BencodingValue::String(s) => s,
            _ => panic!(),
        };
        let number = match list[1] {
            BencodingValue::Integer(i) => i,
            _ => panic!(),
        };

        assert_eq!(elem1, b"elem1");
        assert_eq!(number, 42);
    }

    #[test]
    fn decode_empty_dict_does_not_panic() {
        Bencoding::decode(b"de").unwrap();
    }

    #[test]
    fn get_key_that_does_not_exist_must_return_none() {
        let parser = Bencoding::decode(b"de").unwrap();
        let result = parser.get(b"fake");
        assert!(result.is_none());
    }
}
