use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::fmt::Write;

pub fn markdown_to_telegram_html_chunks(markdown: &str) -> Vec<String> {
    let preprocessed = markdown
        .replace("\\[", "$$")
        .replace("\\]", "$$")
        .replace("\\(", "$")
        .replace("\\)", "$");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);
    let parser = Parser::new_ext(&preprocessed, options);
    
    let mut chunks = Vec::new();
    let mut html = String::new();
    let mut open_tags: Vec<String> = Vec::new();
    let mut close_tags: Vec<String> = Vec::new();

    for event in parser {
        if html.len() > 3800 {
            // Close all open tags
            for tag in close_tags.iter().rev() {
                html.push_str(tag);
            }
            chunks.push(html.clone());
            html.clear();
            // Re-open all open tags
            for tag in open_tags.iter() {
                html.push_str(tag);
            }
        }

        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { .. } | Tag::Strong => {
                    html.push_str("<b>");
                    open_tags.push("<b>".to_string());
                    close_tags.push("</b>".to_string());
                }
                Tag::BlockQuote(_) => {
                    html.push_str("<blockquote>");
                    open_tags.push("<blockquote>".to_string());
                    close_tags.push("</blockquote>".to_string());
                }
                Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                    let tag_str = if lang.is_empty() {
                        "<pre><code>".to_string()
                    } else {
                        format!("<pre><code class=\"language-{}\">", lang)
                    };
                    html.push_str(&tag_str);
                    open_tags.push(tag_str);
                    close_tags.push("</code></pre>".to_string());
                }
                Tag::CodeBlock(CodeBlockKind::Indented) => {
                    html.push_str("<pre><code>");
                    open_tags.push("<pre><code>".to_string());
                    close_tags.push("</code></pre>".to_string());
                }
                Tag::Emphasis => {
                    html.push_str("<i>");
                    open_tags.push("<i>".to_string());
                    close_tags.push("</i>".to_string());
                }
                Tag::Strikethrough => {
                    html.push_str("<s>");
                    open_tags.push("<s>".to_string());
                    close_tags.push("</s>".to_string());
                }
                Tag::Link { dest_url, .. } => {
                    let tag_str = format!("<a href=\"{}\">", dest_url);
                    html.push_str(&tag_str);
                    open_tags.push(tag_str);
                    close_tags.push("</a>".to_string());
                }
                Tag::Item => {
                    html.push_str("• ");
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading { .. } | TagEnd::Strong | TagEnd::BlockQuote(_) | TagEnd::CodeBlock | TagEnd::Emphasis | TagEnd::Strikethrough | TagEnd::Link => {
                    if let Some(close_tag) = close_tags.pop() {
                        html.push_str(&close_tag);
                    }
                    open_tags.pop();
                    
                    match tag {
                        TagEnd::Heading { .. } => html.push_str("\n\n"),
                        TagEnd::BlockQuote(_) | TagEnd::CodeBlock => html.push('\n'),
                        _ => {}
                    }
                }
                TagEnd::Paragraph => html.push_str("\n\n"),
                TagEnd::List(_) | TagEnd::Item => html.push('\n'),
                _ => {}
            },
            Event::Text(text) => {
                let unicode_math = unicodeit::replace(&text);
                html.push_str(&escape_html(&unicode_math));
            }
            Event::Code(text) => {
                let _ = write!(html, "<code>{}</code>", escape_html(&text));
            }
            Event::Html(text) => {
                html.push_str(&escape_html(&text));
            }
            Event::InlineMath(text) => {
                html.push_str(&format!("\\({}\\)", escape_html(&text)));
            }
            Event::DisplayMath(text) => {
                html.push_str(&format!("\\[{}\\]", escape_html(&text)));
            }
            Event::SoftBreak => html.push('\n'),
            Event::HardBreak => html.push_str("\n"),
            _ => {}
        }
    }

    if !html.is_empty() {
        chunks.push(html.trim().to_string());
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    chunks
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
