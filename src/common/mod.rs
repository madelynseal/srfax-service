pub mod winservice;

pub fn set_cwd_to_exe() -> std::io::Result<()> {
    let mut path = std::env::current_exe()?;
    path.pop();

    std::env::set_current_dir(path)?;

    Ok(())
}
