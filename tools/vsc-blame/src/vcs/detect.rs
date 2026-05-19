use crate::blame::VcsKind;
use crate::util::AppError;

pub fn detect_vcs() -> Result<VcsKind, AppError> {
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.as_path();

    loop {
        if dir.join(".git").exists() {
            return Ok(VcsKind::Git);
        }
        if dir.join(".svn").exists() {
            return Ok(VcsKind::Svn);
        }
        dir = match dir.parent() {
            Some(p) => p,
            None => break,
        };
    }

    Err(AppError::vcs_not_found(
        "no VCS detected (no .git or .svn found in any parent directory)",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_in_git_repo() {
        let result = detect_vcs();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), VcsKind::Git);
    }
}
