use unicodeit::replace;
fn main() {
    let md = r#"x^{n-1}"#;
    println!("{}", replace(md));
}
