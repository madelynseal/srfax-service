type Result<T> = std::result::Result<T, anyhow::Error>;

#[cfg(windows)]
pub fn install() -> Result<()> {
    use std::ffi::OsString;
    use windows_service::service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = std::env::current_exe()?;

    let service_info = ServiceInfo {
        name: OsString::from(crate::SERVICE_NAME),
        display_name: OsString::from(crate::SERVICE_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let _service = service_manager.create_service(&service_info, ServiceAccess::empty())?;

    println!("NOTE: service set to run as system");

    Ok(())
}

#[cfg(windows)]
pub fn uninstall() -> Result<()> {
    use std::thread;
    use std::time::Duration;
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(crate::SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(5));
    }

    service.delete()?;
    Ok(())
}

#[cfg(windows)]
pub fn start() -> Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager.open_service(crate::SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Running {
        service.start(&[std::ffi::OsStr::new("Started from Rust!")])?;
    } else {
        println!("already started, state: {:?}", service_status);
        info!("already started, state: {:?}", service_status);
    }

    Ok(())
}

#[cfg(windows)]
pub fn stop() -> Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP;
    let service = service_manager.open_service(crate::SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
    } else {
        println!("already stopped, state: {:?}", service_status);
        info!("already stopped, state: {:?}", service_status);
    }

    Ok(())
}

use clap::{ArgMatches, Command};

#[cfg(windows)]
pub fn add_to_clap(app: Command) -> Command {
    app.subcommand(Command::new("install").about("install windows service"))
        .subcommand(Command::new("uninstall").about("uninstall windows service"))
        .subcommand(Command::new("start").about("start service, uses cmd.exe for now"))
        .subcommand(Command::new("stop").about("stop service"))
}

#[cfg(not(windows))]
pub fn add_to_clap(app: Command) -> Command {
    app
}

#[cfg(windows)]
pub fn check_clap(matches: &ArgMatches) -> Result<bool> {
    if let Some(_matches) = matches.subcommand_matches("install") {
        install()?;

        Ok(true)
    } else if let Some(_matches) = matches.subcommand_matches("uninstall") {
        uninstall()?;

        Ok(true)
    } else if let Some(_matches) = matches.subcommand_matches("start") {
        start()?;

        Ok(true)
    } else if let Some(_matches) = matches.subcommand_matches("stop") {
        stop()?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(not(windows))]
pub fn check_clap(_matches: &ArgMatches) -> Result<bool> {
    Ok(false)
}
