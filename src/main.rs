use std::{error::Error, fs::File, io::Read, collections::HashMap};

use toml::Value;

struct Context {
    functions: HashMap<String, Function>,
}

struct Function {
    arguments: Vec<String>,
    locals: Vec<String>,
    instructions: Vec<Value>,
}

#[derive(Clone)]
enum Data {
    String(String),
    Integer(i64),
    Undefined
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
            function.instructions.push(instruction.clone());
        }

        context.functions.insert(key.clone(), function);
    }

    let main = context.functions.get("main").unwrap();
    execute(&context, main, &Vec::new());

    Ok(())
}

fn evaluate(value: &Value, context: &Context, locals: &HashMap<String, Data>) -> Data {
    match value {
        Value::String(string) => Data::String(string.clone()),
        Value::Integer(integer) => Data::Integer(*integer),
        Value::Table(map) => {
            let (key, value) = map.iter().next().unwrap();

            if locals.contains_key(key) {
                return locals.get(key).unwrap().clone();
            }

            if context.functions.contains_key(key) || key == "print" || key == "println" {
                let arguments = value.as_array().unwrap().clone();
                let arguments_new = arguments.iter().map(|argument| {
                    evaluate(argument, context, locals)
                }).collect();

                match key.as_str() {
                    "print" => print(&arguments_new),
                    "println" => println(&arguments_new),
                    _ => execute(context, context.functions.get(key).unwrap(), &arguments_new)
                }
            }

            Data::Undefined
        }
        _ => Data::Undefined
    }
}

fn execute(context: &Context, function: &Function, arguments: &Vec<Data>) {
    let mut locals: HashMap<String, Data> = HashMap::new();

    for (index, argument) in arguments.iter().enumerate() {
        locals.insert(function.arguments.get(index).unwrap().clone(), argument.clone());
    }

    for instruction in &function.instructions {
        evaluate(instruction, context, &locals);
    }
}

fn println(arguments: &Vec<Data>) {
    print(arguments);
    println!()
}

fn print(arguments: &Vec<Data>) {
    for argument in arguments {
        match argument {
            Data::String(string) => print!("{string}"),
            Data::Integer(integer) => print!("{integer}"),
            Data::Undefined => print!("undefined")
        }
    }
}