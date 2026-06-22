//! OneDrive/SharePoint file export operations over Microsoft Graph.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::exports::client::{GraphClient, GraphOutcome, Sleeper};
use crate::exports::document::build_meeting_docx;
use crate::exports::error::GraphErrorKind;
use crate::exports::transport::{
    GraphBinaryRequest, GraphBinaryResponse, GraphRequest, GraphTransport, TransportError,
};

const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";
pub const DOCX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
pub const PDF_CONTENT_TYPE: &str = "application/pdf";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DriveDestination {
    pub drive_id: String,
    pub item_id: String,
    pub name: String,
    pub web_url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub drive_id: String,
    pub item_id: String,
    pub name: String,
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneDriveExportRequest {
    pub meeting_id: String,
    pub meeting_title: String,
    pub markdown: String,
    #[serde(default)]
    pub transcript: Option<String>,
    #[serde(default)]
    pub destination: Option<DriveDestination>,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub include_pdf: bool,
    #[serde(default)]
    pub create_organization_link: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneDriveExportedFile {
    pub kind: String,
    pub drive_id: String,
    pub item_id: String,
    pub name: String,
    pub web_url: Option<String>,
    pub sharing_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneDriveExportResponse {
    pub destination: DriveDestination,
    pub files: Vec<OneDriveExportedFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphDrive {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    web_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphParentReference {
    #[serde(default)]
    drive_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphRemoteItem {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    web_url: Option<String>,
    #[serde(default)]
    parent_reference: Option<GraphParentReference>,
    #[serde(default)]
    folder: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphDriveItem {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    web_url: Option<String>,
    #[serde(default)]
    parent_reference: Option<GraphParentReference>,
    #[serde(default)]
    folder: Option<serde_json::Value>,
    #[serde(default)]
    remote_item: Option<GraphRemoteItem>,
}

#[derive(Debug, Deserialize)]
struct GraphCreateLinkResponse {
    link: Option<GraphSharingLink>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphSharingLink {
    web_url: Option<String>,
}

fn graph_error(kind: GraphErrorKind, detail: Option<String>) -> String {
    match detail {
        Some(d) => format!("Graph error ({}): {d}", kind.code()),
        None => format!("Graph error: {}", kind.code()),
    }
}

fn graph_failure_detail(error_code: Option<&str>, error_message: Option<&str>) -> Option<String> {
    match (error_code, error_message) {
        (Some(c), Some(m)) => Some(format!("{c}: {m}")),
        (Some(c), None) => Some(c.to_string()),
        (None, Some(m)) => Some(m.to_string()),
        (None, None) => None,
    }
}

fn parse_text_response<T: serde::de::DeserializeOwned>(
    outcome: GraphOutcome,
    what: &str,
) -> Result<T, String> {
    match outcome {
        GraphOutcome::Success(resp) => serde_json::from_str::<T>(&resp.body)
            .map_err(|e| format!("Failed to parse {what}: {e}")),
        GraphOutcome::Failed(kind, detail) => Err(graph_error(kind, detail)),
        GraphOutcome::Unknown(msg) => Err(format!("Network error: {msg}")),
    }
}

fn parse_binary_json_response<T: serde::de::DeserializeOwned>(
    outcome: Result<GraphBinaryResponse, TransportError>,
    what: &str,
) -> Result<T, String> {
    match outcome {
        Ok(resp) if resp.is_success() => serde_json::from_slice::<T>(&resp.body)
            .map_err(|e| format!("Failed to parse {what}: {e}")),
        Ok(resp) => Err(graph_error(
            GraphErrorKind::from_status(resp.status, resp.error_code.as_deref()),
            graph_failure_detail(resp.error_code.as_deref(), resp.error_message.as_deref()),
        )),
        Err(TransportError::Network(msg)) => Err(format!("Network error: {msg}")),
    }
}

fn binary_bytes_response(
    outcome: Result<GraphBinaryResponse, TransportError>,
    what: &str,
) -> Result<Vec<u8>, String> {
    match outcome {
        Ok(resp) if resp.is_success() => Ok(resp.body),
        Ok(resp) => Err(graph_error(
            GraphErrorKind::from_status(resp.status, resp.error_code.as_deref()),
            graph_failure_detail(resp.error_code.as_deref(), resp.error_message.as_deref()),
        )),
        Err(TransportError::Network(msg)) => Err(format!("Network error while {what}: {msg}")),
    }
}

fn get_request(url: String) -> GraphRequest {
    GraphRequest {
        method: "GET".into(),
        url,
        content_type: "application/json".into(),
        body: String::new(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        headers: Vec::new(),
    }
}

fn post_json_request(url: String, body: serde_json::Value) -> GraphRequest {
    GraphRequest {
        method: "POST".into(),
        url,
        content_type: "application/json".into(),
        body: body.to_string(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        headers: Vec::new(),
    }
}

fn binary_request(
    method: impl Into<String>,
    url: String,
    content_type: impl Into<String>,
    body: Vec<u8>,
) -> GraphBinaryRequest {
    GraphBinaryRequest {
        method: method.into(),
        url,
        content_type: content_type.into(),
        body,
        correlation_id: uuid::Uuid::new_v4().to_string(),
        headers: Vec::new(),
    }
}

pub fn encode_sharing_url(sharing_url: &str) -> String {
    format!(
        "u!{}",
        URL_SAFE_NO_PAD.encode(sharing_url.trim().as_bytes())
    )
}

pub fn encode_graph_path_segment(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn sanitize_drive_item_name(raw: &str, fallback: &str) -> String {
    const FORBIDDEN: &[char] = &['"', '*', ':', '<', '>', '?', '/', '\\', '|'];
    let replaced: String = raw
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                ' '
            } else {
                c
            }
        })
        .collect();
    let collapsed = replaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim_matches('.').trim();
    let name = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    name.chars().take(120).collect()
}

fn export_file_stem(meeting_title: &str, meeting_id: &str) -> String {
    let fallback = if meeting_id.trim().is_empty() {
        "Meeting notes"
    } else {
        meeting_id.trim()
    };
    sanitize_drive_item_name(meeting_title, fallback)
}

fn drive_file_from_item(
    item: GraphDriveItem,
    fallback_drive_id: &str,
) -> Result<DriveFile, String> {
    let drive_id = item
        .parent_reference
        .and_then(|p| p.drive_id)
        .unwrap_or_else(|| fallback_drive_id.to_string());
    let name = item.name.unwrap_or_else(|| "Meeting notes".to_string());
    Ok(DriveFile {
        drive_id,
        item_id: item.id,
        name,
        web_url: item.web_url,
    })
}

fn destination_from_item(
    item: GraphDriveItem,
    fallback_drive_id: Option<&str>,
    source: &str,
) -> Result<DriveDestination, String> {
    if let Some(remote) = item.remote_item {
        if remote.folder.is_none() {
            return Err("The sharing URL must point to a folder.".to_string());
        }
        let drive_id = remote
            .parent_reference
            .and_then(|p| p.drive_id)
            .or_else(|| fallback_drive_id.map(str::to_string))
            .ok_or_else(|| "Graph did not return a drive id for the shared folder.".to_string())?;
        return Ok(DriveDestination {
            drive_id,
            item_id: remote.id,
            name: remote.name.unwrap_or_else(|| "Shared folder".to_string()),
            web_url: remote.web_url,
            source: source.to_string(),
        });
    }

    if item.folder.is_none() {
        return Err("The sharing URL must point to a folder.".to_string());
    }
    let drive_id = item
        .parent_reference
        .and_then(|p| p.drive_id)
        .or_else(|| fallback_drive_id.map(str::to_string))
        .ok_or_else(|| "Graph did not return a drive id for the folder.".to_string())?;
    Ok(DriveDestination {
        drive_id,
        item_id: item.id,
        name: item.name.unwrap_or_else(|| "Folder".to_string()),
        web_url: item.web_url,
        source: source.to_string(),
    })
}

pub async fn resolve_default_drive_root<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
) -> Result<DriveDestination, String> {
    let drive: GraphDrive = parse_text_response(
        client
            .execute(
                &get_request(format!("{GRAPH_BASE}/me/drive?$select=id,name,webUrl")),
                token,
            )
            .await,
        "default drive",
    )?;

    let root: GraphDriveItem = parse_text_response(
        client
            .execute(
                &get_request(format!(
                    "{GRAPH_BASE}/me/drive/root?$select=id,name,webUrl,parentReference,folder"
                )),
                token,
            )
            .await,
        "drive root",
    )?;

    let mut destination = destination_from_item(root, Some(&drive.id), "default_drive_root")?;
    if destination.name.eq_ignore_ascii_case("root") {
        if let Some(name) = drive.name {
            if !name.trim().is_empty() {
                destination.name = name;
            }
        }
    }
    if destination.web_url.is_none() {
        destination.web_url = drive.web_url;
    }
    Ok(destination)
}

pub async fn resolve_sharing_url<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    sharing_url: &str,
) -> Result<DriveDestination, String> {
    if sharing_url.trim().is_empty() {
        return Err("Sharing URL is required.".to_string());
    }
    let share_token = encode_sharing_url(sharing_url);
    let request = get_request(format!(
        "{GRAPH_BASE}/shares/{share_token}/driveItem?$select=id,name,webUrl,parentReference,folder,remoteItem"
    ));
    let item: GraphDriveItem =
        parse_text_response(client.execute(&request, token).await, "shared drive item")?;
    destination_from_item(item, None, "sharing_url")
}

pub async fn create_folder<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    parent: &DriveDestination,
    folder_name: &str,
) -> Result<DriveDestination, String> {
    let folder_name = sanitize_drive_item_name(folder_name, "Meeting notes");
    let request = post_json_request(
        format!(
            "{GRAPH_BASE}/drives/{}/items/{}/children",
            parent.drive_id, parent.item_id
        ),
        serde_json::json!({
            "name": folder_name,
            "folder": {},
            "@microsoft.graph.conflictBehavior": "rename",
        }),
    );
    let item: GraphDriveItem =
        parse_text_response(client.execute(&request, token).await, "created folder")?;
    let mut destination = destination_from_item(item, Some(&parent.drive_id), "created_folder")?;
    destination.source = "created_folder".to_string();
    Ok(destination)
}

pub async fn upload_file<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    parent: &DriveDestination,
    file_name: &str,
    content_type: &str,
    bytes: Vec<u8>,
) -> Result<DriveFile, String> {
    let file_name = sanitize_drive_item_name(file_name, "Meeting notes");
    let encoded_name = encode_graph_path_segment(&file_name);
    let request = binary_request(
        "PUT",
        format!(
            "{GRAPH_BASE}/drives/{}/items/{}:/{encoded_name}:/content",
            parent.drive_id, parent.item_id
        ),
        content_type,
        bytes,
    );
    let item: GraphDriveItem = parse_binary_json_response(
        client.transport().send_binary(&request, Some(token)).await,
        "uploaded file",
    )?;
    drive_file_from_item(item, &parent.drive_id)
}

pub async fn convert_docx_to_pdf<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    file: &DriveFile,
) -> Result<Vec<u8>, String> {
    let request = binary_request(
        "GET",
        format!(
            "{GRAPH_BASE}/drives/{}/items/{}/content?format=pdf",
            file.drive_id, file.item_id
        ),
        "",
        Vec::new(),
    );
    binary_bytes_response(
        client.transport().send_binary(&request, Some(token)).await,
        "converting DOCX to PDF",
    )
}

pub async fn create_organization_view_link<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    file: &DriveFile,
) -> Result<String, String> {
    let request = post_json_request(
        format!(
            "{GRAPH_BASE}/drives/{}/items/{}/createLink",
            file.drive_id, file.item_id
        ),
        serde_json::json!({
            "type": "view",
            "scope": "organization",
        }),
    );
    let response: GraphCreateLinkResponse =
        parse_text_response(client.execute(&request, token).await, "sharing link")?;
    response
        .link
        .and_then(|link| link.web_url)
        .ok_or_else(|| "Graph did not return a sharing link.".to_string())
}

pub async fn export_meeting_files<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    request: OneDriveExportRequest,
) -> Result<OneDriveExportResponse, String> {
    let base_destination = match request.destination {
        Some(destination) => destination,
        None => resolve_default_drive_root(client, token).await?,
    };

    let destination = match request.folder_name.as_deref() {
        Some(name) if !name.trim().is_empty() => {
            create_folder(client, token, &base_destination, name).await?
        }
        _ => base_destination,
    };

    let stem = export_file_stem(&request.meeting_title, &request.meeting_id);
    let docx_name = format!("{stem}.docx");
    let docx_bytes = build_meeting_docx(
        &request.meeting_title,
        &request.markdown,
        request.transcript.as_deref(),
    )?;
    let docx = upload_file(
        client,
        token,
        &destination,
        &docx_name,
        DOCX_CONTENT_TYPE,
        docx_bytes,
    )
    .await?;

    let mut files = Vec::new();
    files.push(exported_file(
        "docx",
        &docx,
        maybe_create_link(client, token, &docx, request.create_organization_link).await?,
    ));

    if request.include_pdf {
        let pdf_bytes = convert_docx_to_pdf(client, token, &docx).await?;
        let pdf_name = format!("{stem}.pdf");
        let pdf = upload_file(
            client,
            token,
            &destination,
            &pdf_name,
            PDF_CONTENT_TYPE,
            pdf_bytes,
        )
        .await?;
        files.push(exported_file(
            "pdf",
            &pdf,
            maybe_create_link(client, token, &pdf, request.create_organization_link).await?,
        ));
    }

    Ok(OneDriveExportResponse { destination, files })
}

async fn maybe_create_link<T: GraphTransport, S: Sleeper>(
    client: &GraphClient<T, S>,
    token: &str,
    file: &DriveFile,
    enabled: bool,
) -> Result<Option<String>, String> {
    if enabled {
        Ok(Some(
            create_organization_view_link(client, token, file).await?,
        ))
    } else {
        Ok(None)
    }
}

fn exported_file(
    kind: &str,
    file: &DriveFile,
    sharing_link: Option<String>,
) -> OneDriveExportedFile {
    OneDriveExportedFile {
        kind: kind.to_string(),
        drive_id: file.drive_id.clone(),
        item_id: file.item_id.clone(),
        name: file.name.clone(),
        web_url: file.web_url.clone(),
        sharing_link,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::client::{RetryPolicy, Sleeper};
    use crate::exports::transport::{GraphBinaryResponse, MockGraphTransport};
    use async_trait::async_trait;
    use std::time::Duration;

    #[derive(Default)]
    struct InstantSleeper;

    #[async_trait]
    impl Sleeper for InstantSleeper {
        async fn sleep(&self, _: Duration) {}
    }

    fn client(transport: MockGraphTransport) -> GraphClient<MockGraphTransport, InstantSleeper> {
        GraphClient::new(transport, InstantSleeper, RetryPolicy::default())
    }

    fn parent_destination() -> DriveDestination {
        DriveDestination {
            drive_id: "drive-1".to_string(),
            item_id: "root-id".to_string(),
            name: "OneDrive".to_string(),
            web_url: None,
            source: "default_drive_root".to_string(),
        }
    }

    #[test]
    fn graph_path_segment_encoding_preserves_unreserved_only() {
        assert_eq!(
            encode_graph_path_segment("Meeting notes #1/next.docx"),
            "Meeting%20notes%20%231%2Fnext.docx"
        );
        assert_eq!(encode_graph_path_segment("AZaz09-._~"), "AZaz09-._~");
    }

    #[test]
    fn share_token_encoding_uses_graph_u_prefix_and_base64url() {
        assert_eq!(
            encode_sharing_url("https://example.com/shared"),
            "u!aHR0cHM6Ly9leGFtcGxlLmNvbS9zaGFyZWQ"
        );
        assert!(!encode_sharing_url("https://example.com/shared").contains('='));
    }

    #[tokio::test]
    async fn upload_file_uses_parent_path_content_endpoint() {
        let transport = MockGraphTransport::new();
        transport.queue_binary_default([GraphBinaryResponse::success(
            201,
            br#"{"id":"file-1","name":"Meeting #1.docx","webUrl":"https://example.test/file","parentReference":{"driveId":"drive-1"}}"#
                .to_vec(),
        )]);
        let c = client(transport);

        let file = upload_file(
            &c,
            "token",
            &parent_destination(),
            "Meeting #1.docx",
            DOCX_CONTENT_TYPE,
            b"docx".to_vec(),
        )
        .await
        .unwrap();

        assert_eq!(file.item_id, "file-1");
        let recorded = c.transport().recorded();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].method, "PUT");
        assert_eq!(recorded[0].content_type, DOCX_CONTENT_TYPE);
        assert_eq!(
            recorded[0].url,
            format!("{GRAPH_BASE}/drives/drive-1/items/root-id:/Meeting%20%231.docx:/content")
        );
    }

    #[tokio::test]
    async fn pdf_conversion_uses_format_query_on_uploaded_docx() {
        let transport = MockGraphTransport::new();
        transport.queue_binary_default([GraphBinaryResponse::success(200, b"%PDF-1.7".to_vec())]);
        let c = client(transport);
        let file = DriveFile {
            drive_id: "drive-1".to_string(),
            item_id: "docx-1".to_string(),
            name: "Meeting.docx".to_string(),
            web_url: None,
        };

        let pdf = convert_docx_to_pdf(&c, "token", &file).await.unwrap();

        assert_eq!(pdf, b"%PDF-1.7".to_vec());
        let recorded = c.transport().recorded();
        assert_eq!(recorded[0].method, "GET");
        assert_eq!(
            recorded[0].url,
            format!("{GRAPH_BASE}/drives/drive-1/items/docx-1/content?format=pdf")
        );
    }
}
