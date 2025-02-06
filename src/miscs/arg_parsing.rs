use std::collections::HashMap;

#[derive(Debug)]

pub enum Data {
    None,
    String(String),
    Int(i64),
    Float(f64),
    List(Vec<Data>),
    Object(HashMap<String, Data>),
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.to_string(0))
    }
}

impl Data {
    fn to_string(&self, depth: u64) -> String {
        let mut output = String::new();
        let indent = "\t".repeat(depth as usize);

        match self {
            Data::String(s) => {
                output.push_str(&format!("{}String: '{}'\n", indent, s));
            }
            Data::Int(i) => {
                output.push_str(&format!("{}Int: '{}'\n", indent, i));
            }
            Data::Float(f) => {
                output.push_str(&format!("{}Float: '{}'\n", indent, f));
            }
            Data::List(list) => {
                output.push_str(&format!("{}List:\n", indent));
                for item in list {
                    output.push_str(&item.to_string(depth + 1));
                }
            }
            Data::Object(obj) => {
                output.push_str(&format!("{}Object:\n", indent));
                for (key, value) in obj {
                    output.push_str(&format!(
                        "{}'{}': \n",
                        "\t".repeat((depth + 1) as usize),
                        key
                    ));
                    output.push_str(&value.to_string(depth + 2));
                }
            }
            Data::None => {
                output.push_str(&format!("{}Placeholder\n", indent));
            }
        }

        output
    }

    pub fn print(&self, depth: u64) {
        match self {
            Data::String(s) => println!("{}String: '{}'", "\t".repeat(depth as usize), s),
            Data::Int(i) => println!("{}Int: '{}'", "\t".repeat(depth as usize), i),
            Data::Float(f) => println!("{}Float: '{}'", "\t".repeat(depth as usize), f),
            Data::List(list) => {
                println!("{}List:", "\t".repeat(depth as usize));
                for item in list {
                    item.print(depth + 1);
                }
            }
            Data::Object(obj) => {
                println!("{}Object:", "\t".repeat(depth as usize));
                for (key, value) in obj {
                    println!("{}{}: ", "\t".repeat((depth + 1) as usize), key);
                    value.print(depth + 2);
                }
            }
            Data::None => println!("{}Placeholder", "\t".repeat(depth as usize)),
        }
    }

    pub fn from_string(input: &str) -> Data {
        let trimmed = input.trim();
        // Try to parse as an integer
        if let Ok(int_value) = trimmed.parse::<i64>() {
            return Data::Int(int_value);
        }

        // Try to parse as a float
        if let Ok(float_value) = trimmed.parse::<f64>() {
            return Data::Float(float_value);
        }

        // If parsing fails, return as a string
        Data::String(trimmed.to_string())
    }
}

pub fn parse_data(mut input: &str, x: u64) -> Data {
    input = input.trim();
    input = input.strip_prefix("[").unwrap_or(input);
    input = input.trim();
    input = input.strip_suffix("]").unwrap_or(input);
    input = input.trim();

    if x == 0 {
        println!("first call: {}", input);
    } else {
        println!("call: {} {}", x, input);
    }
    if x > 100 {
        return Data::from_string(input);
    }
    let input = input.trim();
    let mut data: Data = Data::None;
    let mut is_key = false;
    let mut depth = 0;
    let mut current_item = String::new();
    let mut current_key = String::new();
    let mut index = 0;
    for c in input.chars() {
        match c {
            ':' if depth == 0 && current_key.is_empty() => {
                is_key = true;
                // println!("{}", current_item);
                current_key = current_item.trim().to_string().clone();
                current_item.clear();
            }
            ',' if depth == 0 => {
                match data {
                    Data::List(mut list) => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',') || current_item.contains('[') {
                            _data = parse_data(&current_item, x + 1)
                        } else if is_key {
                            let mut object: HashMap<String, Data> = HashMap::new();
                            if current_key.is_empty() {
                                current_key = format!("{}", index);
                                index += 1;
                            }
                            object.insert(current_key.clone(), Data::from_string(&current_item));
                            _data = Data::Object(object);
                        } else {
                            _data = Data::from_string(current_item.trim());
                        }
                        list.push(_data);
                        data = Data::List(list);
                        current_item.clear();
                        current_key.clear();
                    }
                    Data::Object(mut object) => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',') || current_item.contains('[') {
                            _data = parse_data(&current_item, x + 1)
                        } else {
                            _data = Data::from_string(current_item.trim());
                        }

                        if current_key.is_empty() {
                            current_key = format!("{}", index);
                            index += 1;
                        }
                        object.insert(current_key.clone(), _data);
                        data = Data::Object(object);
                        current_key.clear();
                        current_item.clear();
                    }
                    Data::None => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',') || current_item.contains('[') {
                            _data = parse_data(&current_item, x + 1)
                        } else {
                            _data = Data::from_string(current_item.trim());
                        }
                        if is_key {
                            if current_key.is_empty() {
                                current_key = format!("{}", index);
                                index += 1;
                            }
                            let mut hash_map: HashMap<String, Data> = HashMap::new();
                            hash_map.insert(current_key.clone(), _data);
                            data = Data::Object(hash_map);
                        } else {
                            let list: Vec<Data> = vec![_data];
                            //list.push(_data);
                            data = Data::List(list);
                        }
                        current_key.clear();
                        current_item.clear();
                    }
                    _ => {
                        unreachable!()
                    }
                }
                is_key = false;
            }
            '[' => {
                if depth > 0 {
                    current_item.push(c);
                }
                depth += 1;
            }
            ']' => {
                if depth > 0 {
                    current_item.push(c);
                }
                depth -= 1;
            }
            _ => {
                current_item.push(c);
            }
        }
    }
    if !current_item.is_empty() {
        let mut _data: Data = Data::None;
        if current_item.contains(',') || current_item.contains('[') {
            _data = parse_data(&current_item, x + 1)
        } else {
            _data = Data::from_string(current_item.trim());
        }

        match data {
            Data::None => {
                if is_key {
                    if current_key.is_empty() {
                        current_key = format!("{}", index);
                    }
                    let mut object: HashMap<String, Data> = HashMap::new();
                    object.insert(current_key.clone(), _data);
                    data = Data::Object(object);
                } else {
                    let list: Vec<Data> = vec![_data];
                    data = Data::List(list);
                }
            }
            Data::List(mut list) => {
                list.push(_data);
                data = Data::List(list)
            }
            Data::Object(mut object) => {
                if current_key.is_empty() {
                    current_key = format!("{}", index);
                }
                object.insert(current_key.clone(), _data);
                data = Data::Object(object);
            }
            _ => {
                println!("{}, {}, {}", is_key, current_key, current_item);
                unreachable!()
            }
        }
    }
    data
}
