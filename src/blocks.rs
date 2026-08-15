//! Managed blocks: finding, replacing and removing a marker-delimited section inside a
//! file dotflies does not own (ADR 0003).
//!
//! The markers are what make a block *provably ours* — which is why a block can report
//! `drifted` where a plain link cannot. See `plan::classify_link`.

pub fn open_marker(comment: &str, marker: &str) -> String {
    format!("{comment} >>> {marker} >>>")
}

pub fn close_marker(comment: &str, marker: &str) -> String {
    format!("{comment} <<< {marker} <<<")
}

/// The content currently sitting between our markers, if the block is present.
/// `None` means no block; `Some("")` means an empty one.
pub fn find<'a>(text: &'a str, comment: &str, marker: &str) -> Option<&'a str> {
    let open = open_marker(comment, marker);
    let close = close_marker(comment, marker);

    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(text[start..end].trim_matches('\n'))
}

/// Render a whole block, markers included. Used by `apply` once managed blocks land;
/// written now because `find` and `render` have to agree, and a round-trip test is the
/// only way to know they do.
#[allow(dead_code)]
pub fn render(comment: &str, marker: &str, body: &str) -> String {
    format!(
        "{}\n{}\n{}",
        open_marker(comment, marker),
        body.trim_end_matches('\n'),
        close_marker(comment, marker)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZSHRC: &str = "\
export EDITOR=nvim

# >>> dotflies:zsh >>>
export PATH=\"$HOME/.local/bin:$PATH\"
# <<< dotflies:zsh <<<

alias ll='ls -la'
";

    #[test]
    fn finds_our_content_and_nothing_else() {
        let found = find(ZSHRC, "#", "dotflies:zsh").unwrap();
        assert_eq!(found, "export PATH=\"$HOME/.local/bin:$PATH\"");
    }

    #[test]
    fn absent_block_is_none_not_empty() {
        assert_eq!(find(ZSHRC, "#", "dotflies:vim"), None);
    }

    /// An opening marker with no closing one must not swallow the rest of the file.
    #[test]
    fn unterminated_block_reads_as_absent() {
        let truncated = "# >>> dotflies:zsh >>>\nexport PATH=x\n";
        assert_eq!(find(truncated, "#", "dotflies:zsh"), None);
    }

    #[test]
    fn render_round_trips_through_find() {
        let rendered = render("#", "dotflies:zsh", "export PATH=x\n");
        assert_eq!(find(&rendered, "#", "dotflies:zsh"), Some("export PATH=x"));
    }
}
