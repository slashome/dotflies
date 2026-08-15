//! Where things live.
//!
//! There is no configuration file for dotflies itself: the location of the user's
//! configuration is fixed by convention (ADR 0007). Anyone wanting it elsewhere
//! symlinks `~/.config/dotflies` to where they want it.

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

/// The user's configuration repository.
pub fn root() -> Result<Utf8PathBuf> {
    Ok(home()?.join(".config").join("dotflies"))
}

pub fn home() -> Result<Utf8PathBuf> {
    let home = dirs::home_dir().context("could not resolve $HOME")?;
    Utf8PathBuf::from_path_buf(home)
        .map_err(|p| anyhow::anyhow!("$HOME is not valid UTF-8: {}", p.display()))
}

/// Expand a leading `~/` against the real home. Any other form is returned as-is,
/// so an already-absolute path passes through untouched.
pub fn expand(path: &str) -> Result<Utf8PathBuf> {
    match path.strip_prefix("~/") {
        Some(rest) => Ok(home()?.join(rest)),
        None => Ok(Utf8PathBuf::from(path)),
    }
}

/// Render an absolute path back with `~` for display, so reports stay readable.
pub fn contract(path: &Utf8Path) -> String {
    match home() {
        Ok(h) => match path.strip_prefix(&h) {
            Ok(rest) => format!("~/{rest}"),
            Err(_) => path.to_string(),
        },
        Err(_) => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_leaves_absolute_paths_alone() {
        assert_eq!(expand("/etc/hosts").unwrap(), "/etc/hosts");
    }

    #[test]
    fn expand_resolves_tilde() {
        let expanded = expand("~/.config/kitty/kitty.conf").unwrap();
        assert!(expanded.is_absolute());
        assert!(expanded.ends_with(".config/kitty/kitty.conf"));
        assert!(!expanded.as_str().contains('~'));
    }

    /// A bare `~` with no slash is not a path we accept — it stays literal rather than
    /// silently becoming the home directory.
    #[test]
    fn expand_does_not_treat_bare_tilde_as_home() {
        assert_eq!(expand("~").unwrap(), "~");
    }

    #[test]
    fn contract_is_the_inverse_of_expand() {
        let expanded = expand("~/.config/dotflies").unwrap();
        assert_eq!(contract(&expanded), "~/.config/dotflies");
    }
}
