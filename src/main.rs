use std::{error::Error, fs::File, io::Read, collections::HashMap};

use toml::{Value, value::Array};

struct Context {
    functions: HashMap<String, Function>,
}

struct Function {
    arguments: Vec<String>,
    locals: Vec<String>,
    instructions: Vec<Instruction>,
}

enum Instruction {
    INVOKE(String, Value),
    STORE(String, Value)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = "program.toml";
    let mut contents = String::new();
    File::open(file)?.read_to_string(&mut contents)?;

    let test = contents.parse::<Value>()?;
    let functions_toml = test.get("function").unwrap().as_table().unwrap();

    let mut context = Context { functions: HashMap::new() };

    for (key, value) in functions_toml {
        let mut function = Function { arguments: Vec::new(), locals: Vec::new(), instructions: Vec::new() };

        match value.get("locals") {
            Some(locals) => {
                let locals = locals.as_array().unwrap();
                for local in locals {
                    function.locals.push(local.as_str().unwrap().to_string())
                }
            },
            None => ()
        }

        match value.get("arguments") {
            Some(arguments) => {
                let arguments = arguments.as_array().unwrap();
                for argument in arguments {
                    function.locals.push(argument.as_str().unwrap().to_string());
                    function.arguments.push(argument.as_str().unwrap().to_string());
                }
            },
            None => ()
        }

        let instructions = value.get("instructions").unwrap().as_array().unwrap();
        for instruction in instructions {
            let (id, arguments) = instruction.as_table().unwrap().iter().next().unwrap();

            if function.locals.contains(id) {
                function.instructions.push(Instruction::STORE(id.clone(), arguments.clone()))
            } else {
                function.instructions.push(Instruction::INVOKE(id.clone(), arguments.clone()))
            }
        }

        context.functions.insert(key.clone(), function);
    }

    let main = context.functions.get("main").unwrap();
    execute(&context, main, &Vec::new());

    Ok(())
}

fn execute(context: &Context, function: &Function, arguments: &Array) {
    let mut locals: HashMap<String, Value> = HashMap::new();

    for (index, argument) in arguments.iter().enumerate() {
        locals.insert(function.arguments.get(index).unwrap().clone(), argument.clone());
    }

    for instruction in &function.instructions {
        match instruction {
            Instruction::INVOKE(name, arguments) => {
                match name.as_str() {
                    "print" => {
                        let mut arguments = arguments.as_array().unwrap().clone();
                        arguments = arguments.iter().map(|argument| {
                            if let Some(string) = argument.as_str() {
                                if string.chars().nth(0) == Some('$') {
                                    match locals.get(&string[1..]) {
                                        Some(local) => {
                                            return local.clone()
                                        }
                                        None => eprintln!("Local {} not found!", &string[1..])
                                    }
                                }
                            }

                            argument.clone()
                        }).collect();

                        print(&arguments)
                    }
                    "println" => {
                        let mut arguments = arguments.as_array().unwrap().clone();
                        arguments = arguments.iter().map(|argument| {
                            if let Some(string) = argument.as_str() {
                                if string.chars().nth(0) == Some('$') {
                                    match locals.get(&string[1..]) {
                                        Some(local) => {
                                            return local.clone()
                                        }
                                        None => eprintln!("Local {} not found!", &string[1..])
                                    }
                                }
                            }

                            argument.clone()
                        }).collect();

                        println(&arguments)
                    }
                    _ => {
                        let function = context.functions.get(name);

                        let mut arguments = arguments.as_array().unwrap().clone();
                        arguments = arguments.iter().map(|argument| {
                            if let Some(string) = argument.as_str() {
                                if string.chars().nth(0) == Some('$') {
                                    match locals.get(&string[1..]) {
                                        Some(local) => {
                                            return local.clone()
                                        }
                                        None => eprintln!("Local {} not found!", &string[1..])
                                    }
                                }
                            }

                            argument.clone()
                        }).collect();

                        match function {
                            Some(function) => execute(context, function, &arguments),
                            None => eprintln!("Function {name} not defined!")
                        }
                    }
                }
            },
            Instruction::STORE(name, value) => {
                locals.insert(name.clone(), value.clone());
            }
        }
    }
}

fn println(arguments: &Array) {
    print(arguments);
    println!()
}

fn print(arguments: &Array) {
    match arguments.get(0).unwrap() {
        Value::String(string) => print!("{string}"),
        Value::Integer(integer) => print!("{integer}"),
        Value::Float(float) => print!("{float}"),
        _ => {
            eprintln!("Invalid print argument {arguments:?}!");
        }
    }
}
