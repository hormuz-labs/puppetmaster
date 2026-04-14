use opencode_telegram_bot_rs::markdown::markdown_to_telegram_html;

fn main() {
    let md = r#"What is Differentiation?

Differentiation is a tool from calculus used to find the derivative of a function.

• Conceptually: It measures the rate of change of a quantity. For example, if you have a function for distance over time, its derivative tells you the speed.
• Geometrically: It finds the slope of the tangent line to a curve at any given point.

Common Notation

If your original function is f(x) or y, its derivative (the result of differentiation) is written as:

• f'(x) (read as "f prime of x")
• y' (read as "y prime")
• \frac{dy}{dx} (read as "derivative of y with respect to x")

The Most Important Rule: The Power Rule

To differentiate a variable raised to a power (x^n), you bring the exponent down to the front and subtract 1 from the exponent.

Formula: \frac{d}{dx}(x^n) = n \cdot x^{n-1}
"#;
    println!("{}", markdown_to_telegram_html(md));
}
