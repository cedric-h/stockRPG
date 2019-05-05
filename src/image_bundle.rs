use std::collections::HashMap;

const INDEXES_PATH: &str = "./src/data/spritesheet.txt";

pub struct ImageBundle {
    pub map: HashMap<String, [u32; 4]>,
}

impl ImageBundle {
    pub fn new() -> Self {
        use std::fs;

        pub fn parse(filename: &str) -> HashMap<String, [u32; 4]> {
            let contents = fs::read_to_string(filename)
                .expect("Make sure your spritesheet file is named spritesheet.txt");
            let lines = contents.lines();
            let mut map: HashMap<String, [u32; 4]> = HashMap::new();
            for line in lines {
                let words: Vec<&str> = line.split(" ").collect();
                let name = words[0];
                let coords: [u32; 4] = [
                    words[2].parse::<u32>().unwrap(),
                    words[3].parse::<u32>().unwrap(),
                    words[4].parse::<u32>().unwrap(),
                    words[5].parse::<u32>().unwrap(),
                ];
                map.insert(name.to_string(), coords);
            }
            map
        }

        Self {
            map: parse(INDEXES_PATH),
        }
    }
}
