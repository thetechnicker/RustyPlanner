use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Data {
    None,
    String(String),
    Int(i64),
    Float(f64),
    List(Vec<Data>),
    Object(HashMap<String, Data>),
}

#[allow(dead_code)]
impl Data {
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
        // Trim whitespace from the input
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

#[allow(dead_code)]
pub fn parse_data(input: &str, x: u64) -> Data {
    // println!("{}", x);
    if x > 100 {
        return Data::from_string(input);
    }
    let input = input.trim();
    let mut data: Data = Data::None;
    let mut is_key = false;
    let mut depth = 0;
    let mut current_item = String::new();
    let mut current_key = String::new();

    for c in input.chars() {
        match c {
            ':' if depth == 0 => {
                is_key = true;
                // println!("{}", current_item);
                current_key = current_item.trim().to_string().clone();
                current_item.clear();
            }
            ',' if depth == 0 => {
                match data {
                    Data::List(mut list) => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',')
                            || current_item.contains(':')
                            || current_item.contains('[')
                        {
                            _data = parse_data(&current_item, x + 1)
                        } else {
                            if is_key {
                                let mut object: HashMap<String, Data> = HashMap::new();
                                object
                                    .insert(current_key.clone(), Data::from_string(&current_item));
                                _data = Data::Object(object);
                            } else {
                                _data = Data::from_string(&current_item.trim());
                            }
                        }
                        list.push(_data);
                        data = Data::List(list);
                        current_item.clear();
                        current_key.clear();
                    }
                    Data::Object(mut object) => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',')
                            || current_item.contains(':')
                            || current_item.contains('[')
                        {
                            _data = parse_data(&current_item, x + 1)
                        } else {
                            if is_key {
                                let mut object: HashMap<String, Data> = HashMap::new();
                                object
                                    .insert(current_key.clone(), Data::from_string(&current_item));
                                _data = Data::Object(object);
                            } else {
                                _data = Data::from_string(&current_item.trim());
                            }
                        }
                        object.insert(current_key.clone(), _data);
                        data = Data::Object(object);
                        current_key.clear();
                        current_item.clear();
                    }
                    Data::None => {
                        let mut _data: Data = Data::None;
                        if current_item.contains(',')
                            || current_item.contains(':')
                            || current_item.contains('[')
                        {
                            _data = parse_data(&current_item, x + 1)
                        } else {
                            _data = Data::from_string(&current_item.trim());
                        }
                        if is_key {
                            let mut hash_map: HashMap<String, Data> = HashMap::new();
                            hash_map.insert(current_key.clone(), _data);
                            data = Data::Object(hash_map);
                        } else {
                            let mut list: Vec<Data> = Vec::new();
                            list.push(_data);
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
        // println!("{}", x);
    }
    // println!("cba {}: {}", current_key, current_item);
    // println!("--------------");
    if !current_item.is_empty() {
        let mut _data: Data = Data::None;
        if current_item.contains(',') || current_item.contains(':') || current_item.contains('[') {
            _data = parse_data(&current_item, x + 1)
        } else {
            if is_key {
                let mut object: HashMap<String, Data> = HashMap::new();
                object.insert(current_key.clone(), Data::from_string(&current_item));
                _data = Data::Object(object);
            } else {
                _data = Data::from_string(&current_item.trim());
            }
        }

        match data {
            Data::None => {
                if is_key {
                    if current_key.is_empty() {
                        current_key = "aödflja".to_string();
                    }
                    let mut object: HashMap<String, Data> = HashMap::new();
                    object.insert(current_key.clone(), _data);
                    data = Data::Object(object);
                } else {
                    let mut list: Vec<Data> = Vec::new();
                    list.push(_data);
                    data = Data::List(list);
                }
            }
            Data::List(mut list) => {
                list.push(_data);
                data = Data::List(list)
            }
            Data::Object(mut object) => {
                object.insert(current_key.clone(), _data);
                data = Data::Object(object);
            }
            _ => {
                _data.print(0);
                println!("{}, {}, {}", is_key, current_key, current_item);
                unreachable!()
            }
        }
    }
    data
}

#[allow(dead_code)]
pub fn parse_args(input: &str) -> Result<(Vec<String>, HashMap<String, String>), String> {
    let args: Vec<&str> = input.split(',').collect();
    let mut positional_args = Vec::new();
    let mut keyword_args = HashMap::new();
    let mut found_keyword = false; // Flag to track if a keyword argument has been found

    for arg in args {
        let arg = arg.trim(); // Remove any leading/trailing whitespace
        if arg.contains('=') {
            // If we find a keyword argument, set the flag
            found_keyword = true;
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid keyword argument: {}", arg));
            }
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            keyword_args.insert(key, value);
        } else {
            if found_keyword {
                return Err("Positional arguments cannot follow keyword arguments.".to_string());
            }
            positional_args.push(arg.to_string());
        }
    }

    Ok((positional_args, keyword_args))
}
