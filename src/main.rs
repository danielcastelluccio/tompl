use std::{error::Error, fs::File, io::Read, collections::HashMap};

use toml::Value;

struct Context {
    functions: HashMap<String, Function>,
    structs: HashMap<String, Struct>
}

struct Function {
    arguments: Vec<String>,
    locals: Vec<String>,
    instructions: Vec<Value>,
}

struct Struct {
    contents: HashMap<String, Data>,
}

#[derive(Clone, Debug)]
enum Data {
    String(String),
    Integer(i64),
    Struct(HashMap<String, Data>),
    Undefined
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = "program.toml";
    let mut contents = String::new();
    File::open(file)?.read_to_string(&mut contents)?;

    let contents = contents.parse::<Value>()?;

    let mut context = Context { functions: HashMap::new(), structs: HashMap::new() };

    for (key, value) in contents.get("function").unwrap().as_table().unwrap() {
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

    for (key, value) in contents.get("struct").unwrap().as_table().unwrap() {
        let mut struct_ = Struct { contents: HashMap::new() };

        for (key, value) in value.as_table().unwrap() {
            struct_.contents.insert(key.clone(), evaluate(value, None, None, &mut None));
        }

        context.structs.insert(key.clone(), struct_);
    }

    let main = context.functions.get("main").unwrap();
    execute(&context, main, &Vec::new());

    Ok(())
}

fn evaluate(value: &Value, context: Option<&Context>, function: Option<&Function>, locals: &mut Option<&mut HashMap<String, Data>>) -> Data {
    match value {
        Value::String(string) => Data::String(string.clone()),
        Value::Integer(integer) => Data::Integer(*integer),
        Value::Table(map) => {
            let (key, value) = map.iter().next().unwrap();

            if let Some(locals) = locals {
                if function.unwrap().locals.contains(key) {
                    if let Value::Table(table) = value {
                        if table.is_empty() {
                            return locals.get(key).unwrap().clone();
                        }
                    }

                    let evaluated = evaluate(value, context, function, &mut Some(locals));
                    locals.insert(key.clone(), evaluated);
                }
            }

            if let Some(context) = context {
                if context.functions.contains_key(key) || key == "print" || key == "println" {
                    let arguments = value.as_array().unwrap().clone();
                    let mut arguments_new = Vec::new();

                    for argument in &arguments {
                        arguments_new.push(evaluate(argument, Some(context), function, locals))
                    }

                    match key.as_str() {
                        "print" => print(&arguments_new),
                        "println" => println(&arguments_new),
                        _ => execute(context, context.functions.get(key).unwrap(), &arguments_new)
                    }
                }

                if context.structs.contains_key(key) {
                    let mut map = HashMap::new();

                    for (key, value) in &context.structs.get(key).unwrap().contents {
                        map.insert(key.clone(), value.clone());
                    }

                    return Data::Struct(map)
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
        evaluate(instruction, Some(context), Some(function), &mut Some(&mut locals));
    }
}

fn println(arguments: &Vec<Data>) {
    print(arguments);
    println!()
}

fn print(arguments: &Vec<Data>) {
    for argument in arguments {
        print!("{}", to_string(argument));
    }
}

fn to_string(data: &Data) -> String {
    match data {
        Data::String(string) => string.clone(),
        Data::Integer(integer) => integer.to_string(),
        Data::Struct(struct_) => {
            let mut builder = String::new();
            builder += "{";

            for (index, (key, value)) in struct_.iter().enumerate() {
                builder += key;
                builder += " = ";
                builder += &to_string(value);

                if index < struct_.len() - 1 {
                    builder += ", ";
                }
            }

            builder += "}";

            builder
        }
        Data::Undefined => "undefined".to_string(),
    }
}