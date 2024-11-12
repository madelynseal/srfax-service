use crate::{config, EMAIL_SUBJECT_PREFIX};
use chrono::Local;
use futures::Future;
use mail_core::{default_impl::simple_context, Mail};
use mail_headers::header_components::Domain;
use mail_headers::headers::*;
use mail_smtp::{send, ConnectionConfig};
use std::thread;

// forks to a separate thread, allow the program to keep going without hiccup
pub fn send_email_fork(in_subject: &str, in_message: &str) {
    let subject = in_subject.to_owned();
    let message = in_message.to_owned();

    thread::spawn(move || {
        debug!("send_email_fork");
        match send_email(&subject, &message) {
            Ok(()) => (),
            Err(e) => {
                warn!("email error: {}", e);
            }
        }
    });
}

pub fn send_email(in_subject: &str, in_message: &str) -> Result<(), failure::Error> {
    let date_str = Local::now().to_string();
    let subject = format!("{}{}", EMAIL_SUBJECT_PREFIX, in_subject);
    let message = format!("Date: {}\n{}\n", date_str, in_message);

    let config = config::read()?;
    let cemail = config.email;
    if !cemail.enabled {
        return Ok(());
    }

    info!("sending email, subject: {}", subject);

    let con_config = ConnectionConfig::builder_with_addr(
        cemail.server.parse()?,
        mail_smtp::misc::Domain::from_unchecked(cemail.domain.to_owned()),
    )
    .build();

    let ctx = simple_context::new(
        Domain::from_unchecked(cemail.domain.to_owned()),
        cemail.domain.parse()?,
    )?;

    let mut mail = Mail::plain_text(message, &ctx);
    mail.insert_headers(headers! {
        _From: [cemail.from],
        _To: cemail.recipients,
        Subject: subject
    }?);

    let fut = send(mail.into(), con_config, ctx);
    fut.wait()?;

    Ok(())
}
