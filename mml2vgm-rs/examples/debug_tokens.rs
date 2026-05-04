use mml2vgm::compiler::lexer::tokenize;

fn main() {
    // Test various inputs
    println!("=== Song info ===");
    let source = "{ TitleName = MySong }";
    let tokens = tokenize(source).unwrap();
    for (token, pos) in &tokens {
        println!("{}: {:?}", pos, token);
    }
    
    println!("\n=== Basic notes ===");
    let source = "o4 c4 d4 e4 f4";
    let tokens = tokenize(source).unwrap();
    for (token, pos) in &tokens {
        println!("{}: {:?}", pos, token);
    }
    
    println!("\n=== Definition line ===");
    let source = "'A1 T120";
    let tokens = tokenize(source).unwrap();
    for (token, pos) in &tokens {
        println!("{}: {:?}", pos, token);
    }
    
    println!("\n=== Rest ===");
    let source = "r4";
    let tokens = tokenize(source).unwrap();
    for (token, pos) in &tokens {
        println!("{}: {:?}", pos, token);
    }
}
