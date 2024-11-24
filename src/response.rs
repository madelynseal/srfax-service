use core::fmt;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ResultStatus {
    Success,
    Failed,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum DownloadFormat {
    PDF,
    TIF,
}
impl fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Direction {
    IN,
    OUT,
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case, dead_code)]
pub struct InboxItem {
    pub FileName: String,
    pub ReceiveStatus: String,
    pub Date: String,
    pub CallerID: String,
    pub RemoteID: String,
    pub Pages: String,
    pub Size: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct Inbox {
    pub Status: ResultStatus,
    pub Result: Option<Vec<InboxItem>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct RetrieveFaxResponse {
    pub Status: ResultStatus,
    pub Result: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct DeleteFaxResponse {
    pub Status: ResultStatus,
    pub Result: String, // error message
}
