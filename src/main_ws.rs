use crate::email;
use crate::Result;
use crate::{EXIT_CODE_0, SERVICE_NAME};
use std::ffi::OsString;
use std::sync::mpsc;
use std::time::Duration;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, my_service_main);
pub fn main() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;

    Ok(())
}

fn my_service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(arguments) {
        panic!("error: {:?}", e);
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,
        // The new state
        current_state: ServiceState::Running,
        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: Duration::default(),
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;

    std::thread::spawn(|| {
        // start thread
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                ServiceControl::Stop => {
                    match email::send_email("shutting down", "srfax shutting down") {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Error sending shutdown email! {0:?}", e);
                        }
                    }

                    unwrap!(shutdown_tx.send(()));
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
            .expect("could not create status handle");

        loop {
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    status_handle
                        .set_service_status(ServiceStatus {
                            service_type: ServiceType::OWN_PROCESS,
                            current_state: ServiceState::Stopped,
                            controls_accepted: ServiceControlAccept::empty(),
                            exit_code: ServiceExitCode::Win32(0),
                            checkpoint: 0,
                            wait_hint: Duration::default(),
                            process_id: None,
                        })
                        .expect("error setting status");
                    info!("shutting down!");
                    std::process::exit(EXIT_CODE_0);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            };
        }
    }); // end thread

    crate::run_program()?;

    Ok(())
}
