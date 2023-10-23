const INNER_COMMANDS: [&str; 6] = [
    "exit", // exit the program 
    "open", // open a file @open <file>
    "exp", // evaluate a mathematical expression @exp <expression>
    "help", // show help
    "dico", // search in the dictionnary
    "wiki" // search on wikipedia
];

pub fn exec(input: String) {
    if input.is_empty() {
        return;
    }
    let input_type = detect_input_type(&input);
    println!("{:?}", input_type);
}

#[derive(Debug)]
enum InputType {
    Search,
    Mathematical,
    Url,
    Command(String),
    File(String),
}


fn detect_input_type(input: &str) -> InputType {
    if input.starts_with("file://") {
        return InputType::File(input[7..].to_string());
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        return InputType::Url;
    }
    if input.starts_with("@") {
        let command = input[1..].to_string();
        let mut split = command.split(" ");
        let cmd = split.next().unwrap();
        if INNER_COMMANDS.contains(&cmd) {
            return InputType::Command(cmd.to_string());
        }
        return InputType::Search;
        
    }
    match exmex::eval_str::<f64>(input) {
        Ok(_) => {
            return InputType::Mathematical;
        },
        Err(_) => {
            return InputType::Search;
        },
    }
}
