use crate::utils;
use exmex::ExError;
use relm4::gtk::prelude::*;
use relm4::gtk;

// path things
use std::path::PathBuf;
use std::fs;

const INNER_COMMANDS: [&str; 6] = [
    "exit", // exit the program 
    "open", // open a file @open <file>
    "exp", // evaluate a mathematical expression @exp <expression>
    "help", // show help
    "dico", // search in the dictionnary
    "wiki" // search on wikipedia
];

const HELP: &str = r#"    
    @exit: exit the program
    @open <file>: open a file
    @exp <expression>: evaluate a mathematical expression
    @help: show help
    @dico <word>: search in the dictionnary
    @wiki <word>: search on wikipedia
"#;

#[derive(Debug)]
pub enum Action {
    Open(String),
    Exit,
}

#[derive(Debug)]
pub struct Result {
    pub action: Option<Action>,
    pub data: String,
    pub components: Vec<gtk::Box>,
    // A component is a Box containing a Label named "Title" and a Box named "Content"
}

pub fn exec(input: String) -> Result {
    if input.is_empty() {
        return Result {
            action: None,
            data: input,
            components: Vec::new(),
        }
    }
    let input_type = detect_input_type(&input);
    match input_type {
        InputType::Search => {
            search(input)
        },
        InputType::Mathematical => {
            mathematical(input)
        },
        InputType::Url => {
            url(input)
        },
        InputType::Command(cmd) => {
            command(cmd.as_str(), input)
        },
        InputType::File(file) => {
            f(file)
        }
    }
}

fn get_section_title(label: String) -> gtk::Label {
    gtk::Label::builder()
        .name("SectionTitle")
        .css_name("SectionTitle")
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(label)     
        .build()
}

fn search(input: String) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title("Search".to_string());
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .css_classes(vec!["search"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(format!("search {}", input))        
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&title);
    box_content.append(&content);
    components.push(box_content);

    Result {
        action: None, 
        data: input,
        components,
    }
}

fn mathematical(input: String) -> Result {
    if input.is_empty() {
        return Result {
            action: None,
            data: input,
            components: Vec::new(),
        }
    }
    let result = exmex::eval_str::<f64>(&input);
    match result {
        Ok(result) => {
            return mathematical_result(input, result);
        },
        Err(e) => {
            return mathematical_error(input, e);
        },
    }
}

fn mathematical_result(input: String, result: f64) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title(format!("({}) evaluation", input.clone().trim_start()));
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .css_classes(vec!["mathematical"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(result.to_string())     
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&title);
    box_content.append(&content);
    components.push(box_content);

    Result {
        action: None, 
        data: input,
        components,
    }
}

fn mathematical_error(input: String, err: ExError) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title(format!("({}) evaluation", input.clone().trim_start()));
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .css_classes(vec!["matherror"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(format!("{}", err))     
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&title);
    box_content.append(&content);
    components.push(box_content);

    Result {
        action: None, 
        data: input,
        components,
    }
}

fn url(input: String) -> Result { 
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title("Url".to_string());
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .css_classes(vec!["url"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(format!("open {}", input))        
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();

    box_content.append(&title);
    box_content.append(&content);
    
    components.push(box_content);

    Result {
        action: Some(Action::Open(input.clone())),
        data: input,
        components,
    }
}

fn command(cmd: &str, input: String) -> Result {
    match cmd {
        "exit" => {
            Result {
                action: Some(Action::Exit),
                data: input,
                components: Vec::new(),
            }
        },

        "open" => {
            let file = input[5..].to_string();
            let mut components: Vec<gtk::Box> = Vec::new();
            let title = get_section_title("Open".to_string());
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .css_classes(vec!["open"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(format!("open {}", file))        
                .build();

            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            box_content.append(&title);
            box_content.append(&content);
            
            components.push(box_content);

            Result {
                action: Some(Action::Open(file.clone())),
                data: input,
                components,
            }
        },

        "exp" => {
            mathematical(input[4..].to_string())
        },

        "help" => {
            let mut components: Vec<gtk::Box> = Vec::new();
            let title = get_section_title("Help".to_string());
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .css_classes(vec!["help"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(HELP)        
                .build();

            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            box_content.append(&title);
            box_content.append(&content);
            
            components.push(box_content);

            Result {
                action: None,
                data: input,
                components,
            }
        },

        "dico" => {
            // find in the dictionnary
            Result {
                action: None,
                data: input,
                components: Vec::new(),
            }
        },
        "wiki" => {
            // search on wikipedia
            Result {
                action: None,
                data: input,
                components: Vec::new(),
            }
        },

        _ => {
            Result {
                action: None,
                data: input,
                components: Vec::new(),
            }
        },

    }
}

fn f(file: String) -> Result {
    // find in the file system
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title("File".to_string());

    // check if the file exists
    let path = PathBuf::from(file.clone());
    if !path.exists() {
        let content = gtk::Label::builder()
            .name("Content")
            .css_name("Content")
            .css_classes(vec!["file"])
            .hexpand(true)
            .halign(gtk::Align::Start)
            .label(format!("{} does not exist.", file))        
            .build();

        let box_content = gtk::Box::builder()
            .name("BoxContent")
            .css_name("BoxContent")
            .orientation(gtk::Orientation::Vertical)
            .build();

        box_content.append(&title);
        box_content.append(&content);
        
        components.push(box_content);

        return Result {
            action: None,
            data: file,
            components,
        }
    }

    // check if the file is a directory
    let metadata = fs::metadata(path.clone());
    match metadata {
        Ok(meta) => {
            // is directory -> directory_path size last_modified
            if meta.is_dir() {
                let directory_name = gtk::Label::builder()
                    .name("DirectoryName")
                    .css_name("DirectoryName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.display().to_string()))
                    .build();

                let directory_size = gtk::Label::builder()
                    .name("DirectorySize")
                    .css_name("DirectorySize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();
                
                let directory_last_modified = gtk::Label::builder()
                    .name("DirectoryLastModified")
                    .css_name("DirectoryLastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_dir = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_dir.append(&directory_name);
                box_dir.append(&directory_size);
                box_dir.append(&directory_last_modified);
                
                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_dir);

                components.push(box_content);

                return Result {
                    action: None,
                    data: file,
                    components,
                }

            }

            // is file -> file_name size file_type last_modified
            else if meta.is_file() {
                let file_name = gtk::Label::builder()
                    .name("FileName")
                    .css_name("FileName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.file_name().unwrap().to_str().unwrap()))
                    .build();

                let file_size = gtk::Label::builder()
                    .name("FileSize")
                    .css_name("FileSize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();

                let file_type = gtk::Label::builder()
                    .name("FileType")
                    .css_name("FileType")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{:?}", meta.file_type()))
                    .build();

                let last_modified = gtk::Label::builder()
                    .name("LastModified")
                    .css_name("LastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_file = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_file.append(&file_name);
                box_file.append(&file_size);
                box_file.append(&file_type);
                box_file.append(&last_modified);

                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_file);

                components.push(box_content);

                return Result {
                    action: None,
                    data: file,
                    components,
                }
            }
            
            // is symlink -> symlink_name size file_type last_modified
            else if meta.is_symlink() {
                let symlink_name = gtk::Label::builder()
                    .name("SymlinkName")
                    .css_name("SymlinkName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.file_name().unwrap().to_str().unwrap()))
                    .build();

                let symlink_size = gtk::Label::builder()
                    .name("SymlinkSize")
                    .css_name("SymlinkSize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();

                let symlink_type = gtk::Label::builder()
                    .name("SymlinkType")
                    .css_name("SymlinkType")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{:?}", meta.file_type()))
                    .build();

                let last_modified = gtk::Label::builder()
                    .name("LastModified")
                    .css_name("LastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_symlink = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_symlink.append(&symlink_name);
                box_symlink.append(&symlink_size);
                box_symlink.append(&symlink_type);
                box_symlink.append(&last_modified);

                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_symlink);

                components.push(box_content);

                return Result {
                    action: None,
                    data: file,
                    components,
                }

            } else {
               return Result {
                    action: None,
                    data: file,
                    components,
               }
            }


        },
        Err(_) => {
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .css_classes(vec!["file"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(format!("Cannot get metadata of {}", file))        
                .build();
            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            box_content.append(&title);
            box_content.append(&content);
            
            return Result {
                action: None,
                data: file,
                components,
            }
        },
    }
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
