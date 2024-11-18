use crate::{
    config::{self, Srfax},
    email,
    response::*,
    srfax,
};
use reqwest::blocking::Client;
use std::thread;
use std::time;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SrfaxService(Srfax({0:?}))")]
    Srfax(#[from] crate::srfax::SrfaxError),

    #[error("SrfaxService(failed to get inbox)")]
    FailedToGetInbox,

    #[error("SrfaxService(could not connect to srfax)")]
    NoConnection,
}
type Result<T> = std::result::Result<T, Error>;

pub fn run_srfax_service(tick_time: time::Duration) {
    loop {
        thread::spawn(move || {
            let srfaxes = config::get_srfaxes().unwrap();

            for srfax in srfaxes {
                thread::spawn(move || match run_srfax_single(&srfax) {
                    Ok(()) => {
                        info!("updated srfax! name={}", srfax.name);
                    }
                    Err(e) => {
                        warn!("error running srfax! {:?} {:?}", srfax, e);

                        email::send_email_fork(
                            "error running",
                            &format!("error running srfax! {:?} {:?}", srfax, e),
                        );
                    }
                });
            }
        });

        thread::sleep(tick_time);
    }
}

fn run_srfax_single(srfax: &Srfax) -> Result<()> {
    let client = Client::new();

    if !srfax::test_connection(&client) {
        return Err(Error::NoConnection);
    }

    let result = srfax::get_fax_inbox(&client, srfax)?;

    if result.Status != ResultStatus::Success {
        return Err(Error::FailedToGetInbox);
    }

    if result.Result.is_none() {
        return Ok(());
    }
    let inbox = result.Result.unwrap();

    for item in inbox {
        debug!("srfax item: {:?}", item);

        match srfax::retrieve_fax(&client, srfax, &item, Direction::IN) {
            Ok(()) => (),
            Err(e) => {
                warn!("error retrieving fax! item={:?} error={:?}", item, e);
                email::send_email_fork(
                    "error retrieving fax",
                    &format!("error retrieving fax! item={:?} error={:?}", item, e),
                );
                continue;
            }
        }

        if srfax.delete_after {
            match srfax::delete_fax(&client, srfax, &item, Direction::IN) {
                Ok(resp) => {
                    if resp.Status != ResultStatus::Success {
                        warn!("error deleting fax! msg={}", resp.Result);
                    }
                }
                Err(e) => {
                    warn!(
                        "error deleting fax! FileName=[{}] RemoteID=[{}] error={:?}",
                        item.FileName, item.RemoteID, e
                    );
                    email::send_email_fork(
                        "error deleting fax",
                        &format!(
                            "error deleting fax! FileName=[{}] RemoteID=[{}] error={:?}",
                            item.FileName, item.RemoteID, e
                        ),
                    );
                    continue;
                }
            }
        }
    }

    Ok(())
}
