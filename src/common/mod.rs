/* requires crate::EMAIL_SUBJECT_PREFIX: &str to be set */
pub mod email;
//pub mod phone;
//pub mod sql;
/* requres crate::SERVICE_NAME: &str to be set */
//pub mod ntlm;
pub mod winservice;

pub fn set_cwd_to_exe() -> std::io::Result<()> {
    let mut path = std::env::current_exe()?;
    path.pop();

    std::env::set_current_dir(path)?;

    Ok(())
}
