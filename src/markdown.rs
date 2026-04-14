use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use std::fmt::Write;

pub fn markdown_to_telegram_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { .. } => {
                    html.push_str("<b>");
                }
                Tag::BlockQuote(_) => {
                    html.push_str("<blockquote>");
                }
                Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                    if lang.is_empty() {
                        html.push_str("<pre><code>");
                    } else {
                        let _ = write!(html, "<pre><code class=\"language-{}\">", lang);
                    }
                }
                Tag::CodeBlock(CodeBlockKind::Indented) => {
                    html.push_str("<pre><code>");
                }
                Tag::List(_) => {}
                Tag::Item => {
                    html.push_str("• ");
                }
                Tag::Strong => html.push_str("<b>"),
                Tag::Emphasis => html.push_str("<i>"),
                Tag::Strikethrough => html.push_str("<s>"),
                Tag::Link { dest_url, .. } => {
                    let _ = write!(html, "<a href=\"{}\">", dest_url);
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    html.push_str("\n\n");
                }
                TagEnd::Heading { .. } => {
                    html.push_str("</b>\n\n");
                }
                TagEnd::BlockQuote(_) => {
                    html.push_str("</blockquote>\n");
                }
                TagEnd::CodeBlock => {
                    html.push_str("</code></pre>\n");
                }
                TagEnd::List(_) => {
                    html.push('\n');
                }
                TagEnd::Item => {
                    html.push('\n');
                }
                TagEnd::Strong => html.push_str("</b>"),
                TagEnd::Emphasis => html.push_str("</i>"),
                TagEnd::Strikethrough => html.push_str("</s>"),
                TagEnd::Link => html.push_str("</a>"),
                _ => {}
            },
            Event::Text(text) => {
                html.push_str(&escape_html(&text));
            }
            Event::Code(text) => {
                let _ = write!(html, "<code>{}</code>", escape_html(&text));
            }
            Event::Html(text) => {
                // We should probably escape HTML in markdown to prevent Telegram parser errors
                html.push_str(&escape_html(&text));
            }
            Event::SoftBreak => html.push('\n'),
            Event::HardBreak => html.push_str("\n"),
            _ => {}
        }
    }

    html.trim().to_string()
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
