use crate::{config::Srfax, response::*};
use reqwest::blocking::{Client, Response};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum SrfaxError {
    #[error("Srfax(Reqwest({0:?}))")]
    Reqwest(#[from] reqwest::Error),

    #[error("Srfax(IO({0:?}))")]
    Io(#[from] std::io::Error),

    #[error("Srfax(Base64({0:?}))")]
    Base64(#[from] base64::DecodeError),

    #[error("Srafx(possible directory traversal attack! filename={0})")]
    DirectoryTraversal(String),

    #[error("Srfax(failed to download item {0:?})")]
    FailedToDownload(Box<InboxItem>),
}

type Result<T> = std::result::Result<T, SrfaxError>;

pub const SRFAX_ROOT: &str = "https://www.srfax.com";
pub const SRFAX_API: &str = "https://www.srfax.com/SRF_SecWebSvc.php";

pub const SRFAX_ACTION_GET_INBOX: &str = "Get_Fax_Inbox";
pub const SRFAX_ACTION_RETRIEVE: &str = "Retrieve_Fax";
pub const SRFAX_ACTION_DELETE: &str = "Delete_Fax";

pub fn test_connection(client: &Client) -> bool {
    match client.get(SRFAX_ROOT).send() {
        Ok(resp) => resp.status().is_success(),
        Err(e) => {
            warn!("could not connect to srfax! {}", e);
            false
        }
    }
}

pub fn get_fax_inbox(client: &Client, srfax: &Srfax) -> Result<Inbox> {
    let data = vec![("sPeriod", "ALL")];

    let resp = send_post(client, SRFAX_ACTION_GET_INBOX, data, srfax)?;

    let obj: Inbox = resp.json()?;
    Ok(obj)
}

pub fn retrieve_fax(
    client: &Client,
    srfax: &Srfax,
    item: &InboxItem,
    direction: Direction,
) -> Result<()> {
    let direction = direction.to_string();
    let download_fmt = srfax.download_fmt.to_string();

    let (filename, _details_id) = split_fax_filename(&item.FileName);
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(SrfaxError::DirectoryTraversal(filename.to_string()));
    }
    let filepath = {
        let mut path = PathBuf::from(srfax.file_dir.to_owned());

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        path.push(filename);
        path.set_extension(srfax.download_fmt.to_string());

        path
    };

    if filepath.exists() {
        debug!("{:?} already exists, skipping", filepath);
        return Ok(());
    }

    let data = vec![
        ("sFaxFileName", item.FileName.as_str()),
        ("sDirection", direction.as_str()),
        ("sFaxFormat", &download_fmt),
    ];

    let resp = send_post(client, SRFAX_ACTION_RETRIEVE, data, srfax)?;
    let result: RetrieveFaxResponse = resp.json()?;

    if result.Status != ResultStatus::Success {
        return Err(SrfaxError::FailedToDownload(Box::new(item.clone())));
    }
    let base64_data = unwrap!(result.Result).replace('\n', "");

    let file_data = base64::decode(&base64_data)?;

    write_to_file(&file_data, &filepath)?;

    Ok(())
}

pub fn delete_fax(
    client: &Client,
    srfax: &Srfax,
    item: &InboxItem,
    direction: Direction,
) -> Result<DeleteFaxResponse> {
    let direction = direction.to_string();

    let (_filename, details_id) = split_fax_filename(&item.FileName);

    let data = vec![
        ("sDirection", direction.as_str()),
        ("sFaxFilename_x", item.FileName.as_str()),
        ("sFaxDetailsID_x", details_id),
    ];

    let resp = send_post(client, SRFAX_ACTION_DELETE, data, srfax)?;
    let result: DeleteFaxResponse = resp.json()?;

    Ok(result)
}

fn write_to_file(data: &[u8], dest: &Path) -> std::io::Result<()> {
    let mut file = File::create(dest)?;

    file.write_all(data)?;
    Ok(())
}

fn split_fax_filename(s: &str) -> (&str, &str) {
    let index = unwrap!(s.find('|'));

    s.split_at(index)
}

fn send_post<'a>(
    client: &Client,
    action: &'a str,
    mut data: Vec<(&str, &'a str)>,
    srfax: &'a Srfax,
) -> Result<Response> {
    data.push(("action", action));
    data.push(("access_id", &srfax.access_id));
    data.push(("access_pwd", &srfax.access_pwd));

    let resp = client.post(SRFAX_API).form(&data).send()?;

    Ok(resp)
}
