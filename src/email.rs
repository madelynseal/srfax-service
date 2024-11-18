use crate::{config::CONFIG, EMAIL_SUBJECT_PREFIX};
use chrono::Local;
use lettre::{
    message::header::ContentType, transport::smtp::client::Tls, Message, SmtpTransport, Transport,
};
use std::thread;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Email(Lettre({0:?}))")]
    Lettre(#[from] lettre::error::Error),

    #[error("Email(LettreSmtp({0:?}))")]
    LettreSmtp(#[from] lettre::transport::smtp::Error),
}
type Result<T> = std::result::Result<T, EmailError>;

pub fn send_email_fork(in_subject: &str, in_message: &str) {
    let subject: String = in_subject.to_owned();
    let message: String = in_message.to_owned();
    thread::spawn(move || send_email(&subject, &message));
}

pub fn send_email(in_subject: &str, in_message: &str) -> Result<()> {
    if !CONFIG.email.enabled {
        debug!("email not enabled, not sending");
        return Ok(());
    }

    let date_str = Local::now().to_string();
    let subject = format!("{}{}", EMAIL_SUBJECT_PREFIX, in_subject);
    let message = format!("Date: {}\n{}\n", date_str, in_message);

    let mailer = SmtpTransport::starttls_relay(&CONFIG.email.server)
        .unwrap()
        .port(CONFIG.email.port)
        .tls(Tls::None)
        .build();

    for recipient in &CONFIG.email.recipients {
        send_email_single(&subject, &message, &CONFIG.email.from, &recipient, &mailer)?;
    }

    Ok(())
}

pub fn send_email_single(
    subject: &str,
    msg: &str,
    from: &str,
    to: &str,
    mailer: &SmtpTransport,
) -> Result<()> {
    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(String::from(msg))?;

    mailer.send(&email)?;

    Ok(())
}
