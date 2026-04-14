use pulldown_cmark::{Parser, Options, Event};

fn main() {
    let md = r#"\frac{d}{dx}(x^n) = n \cdot x^{n-1}"#;
    let parser = Parser::new_ext(md, Options::empty());
    for e in parser {
        if let Event::Text(t) = e {
            println!("{}", t);
        }
    }
}
