use anyhow::Result;
use dialoguer::console::Term;
use dialoguer::Confirm;

/// Prompt user to reply with YES or NO
pub(crate) fn confirm(question: &str) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(question)
        .interact_on(&Term::stderr())
        .map_err(anyhow::Error::new)?;
    Ok(confirmed)
}
