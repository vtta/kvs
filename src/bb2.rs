use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Move {
    distance: f32,
    direction: Direction,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Direction {
    x: f32,
    y: f32,
    z: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use rand::Rng;
    use ron;
    use std::{fs, io, io::Write};

    impl Move {
        fn new(x: f32, y: f32, z: f32, d: f32) -> Self {
            Move {
                distance: d,
                direction: Direction { x, y, z },
            }
        }
    }

    #[test]
    fn test_json() {
        let mut rng = rand::thread_rng();
        let file = "serde.json";
        let a = Move::new(rng.gen(), rng.gen(), rng.gen(), rng.gen());
        {
            let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file)
                .unwrap();
            let mut writer = io::BufWriter::new(file);
            writer
                .write_all(serde_json::to_string(&a).unwrap().as_bytes())
                .unwrap();
        }
        let b = {
            let file = fs::OpenOptions::new().read(true).open(&file).unwrap();
            let reader = io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };
        fs::remove_file(file).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_ron() {
        let mut rng = rand::thread_rng();
        let file = "serde.ron";
        let a = Move::new(rng.gen(), rng.gen(), rng.gen(), rng.gen());
        {
            let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file)
                .unwrap();
            let mut writer = io::BufWriter::new(file);
            writer
                .write_all(ron::ser::to_string(&a).unwrap().as_bytes())
                .unwrap();
        }
        let b: Move = {
            let file = fs::OpenOptions::new().read(true).open(&file).unwrap();
            let reader = io::BufReader::new(file);
            ron::de::from_reader(reader).unwrap()
        };
        fs::remove_file(file).unwrap();
        assert_eq!(a, b);

        let vec = ron::ser::to_string(&a).unwrap().into_bytes();
        let b: Move = ron::de::from_bytes(&vec).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_bson() {
        let mut rng = rand::thread_rng();
        let file = "serde.bson";
        let mut vec = Vec::new();
        for _ in 0..10000 {
            vec.push(Move::new(rng.gen(), rng.gen(), rng.gen(), rng.gen()));
        }
        {
            let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file)
                .unwrap();
            let mut writer = io::BufWriter::new(file);
            for i in &vec {
                if let Ok(bson::Bson::Document(doc)) = bson::to_bson(i) {
                    bson::encode_document(&mut writer, &doc).unwrap();
                }
            }
        }
        let mut dvec = Vec::new();
        {
            let file = fs::OpenOptions::new().read(true).open(&file).unwrap();
            let mut reader = io::BufReader::new(file);
            while let Ok(doc) = bson::decode_document(&mut reader) {
                let x: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
                dvec.push(x);
            }
        }
        fs::remove_file(file).unwrap();
        assert_eq!(vec, dvec);

        let mut buf = Vec::new();
        let mut dvec = Vec::new();
        {
            for i in &vec {
                if let Ok(bson::Bson::Document(doc)) = bson::to_bson(i) {
                    bson::encode_document(&mut buf, &doc).unwrap();
                }
            }
        }
        let mut cursor = io::Cursor::new(buf);
        {
            while let Ok(doc) = bson::decode_document(&mut cursor) {
                let x: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
                dvec.push(x);
            }
        }
        assert_eq!(vec, dvec);
    }
}
