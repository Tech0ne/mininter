use mininter::prelude::*;

fn print_code(code: &str) {
    for (i, line) in code.lines().enumerate() {
        println!("{:3} | {}", i + 1, line);
    }
}

fn main() {
    let code = r#"x = 12
print x
x += 5
print x + 2

Entity = {}
Entity.name = "[unspecified]"
Entity.die = function()
    print self.name + " is dieing..."
end function

Human = new Entity
Human.name = "Human"

bob = new Human
bob.name = "bob"
bob.die

create_human = function(get_name)
    human = new Human
    human.name = get_name
    return human
end function

get_bub_name = function()
    return "bub"
end function

bub = create_human(@get_bub_name)

print bub isa Human
"#;

    println!("Scanning the following code:\n=============");

    print_code(code);

    println!("=============");

    let scanner = Scanner::new(code);

    let mut line = 0;

    for token in scanner {
        match token {
            Ok(mut t) => {
                t.set_prev_line(line);
                line = t.line();
                println!("{}", t);
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }
}
