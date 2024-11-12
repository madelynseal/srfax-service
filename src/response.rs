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
impl std::string::ToString for DownloadFormat {
    fn to_string(&self) -> String {
        use self::DownloadFormat::*;
        match self {
            PDF => String::from("PDF"),
            TIF => String::from("TIF"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Direction {
    IN,
    OUT,
}
impl std::string::ToString for Direction {
    fn to_string(&self) -> String {
        use self::Direction::*;
        match self {
            IN => String::from("IN"),
            OUT => String::from("OUT"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
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
