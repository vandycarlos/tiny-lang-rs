#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Int(i64),
    Float(f64),
    Rational(Ratio),
    String(String),
    Symbol(String, String),
    Keyword(String, String),
    ListParen(Vec<Value>),
    ListBracket(Vec<Value>),
    ListBrace(Vec<Value>),
}

impl Value {
    pub fn dump(&self, ident: &str) {
        match self {
            Value::String(v) => println!("{ident}'{v}' string"),
            Value::Symbol(_, v) => println!("{ident}'{v}' symbol"),
            Value::Keyword(_, v) => println!("{ident}'{v}' keyword"),

            Value::Int(v) => println!("{ident}'{v}' int"),
            Value::Float(v) => println!("{ident}'{v}' float"),
            Value::Rational(rat) => println!("{ident}'{}/{}' rational", rat.numer, rat.denom),

            Value::ListParen(list) => dump_list(list, ident, '(', ')'),
            Value::ListBracket(list) => dump_list(list, ident, '[', ']'),
            Value::ListBrace(list) => dump_list(list, ident, '{', '}'),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Ratio {
    numer: i64,
    denom: i64,
}

impl Ratio {
    pub fn new(numer: i64, denom: i64) -> Ratio {
        Ratio { numer, denom }
    }
}

fn dump_list(list: &Vec<Value>, ident: &str, left: char, right: char) {
    println!("{ident}{left}");
    for item in list {
        item.dump(format!("  {ident}").as_str());
    }
    println!("{ident}{right}");
}
