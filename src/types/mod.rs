#[test]
macro_rules! setup_test(
    ($filename:expr, $code:expr) => {
        match BufferedReader::new(File::open(&Path::new($filename))).read_line() {
            Ok(line) => {
                println!("{}", line.as_slice());
                match json::decode(line.as_slice()) {
                    Ok(res) => {
                        $code(res)
                    },
                    Err(err) => {
                        println!("{}", err);
                        assert!(false);
                    }
                };
            },
            Err(err) => {
                println!("{}", err);
                assert!(false);
            }
        }
    }
);

macro_rules! json_field {
    ($name:expr, $decoder:expr) => {
        try!($decoder.read_struct_field($name, 0, |d| Decodable::decode(d)))
    }
}

pub mod recording;
pub mod folder;
