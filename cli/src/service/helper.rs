use super::{Pica, Printer};
use crate::domain::{CHECK_INTERNET_CONNECTION_SUG, CHECK_PARAMETERS_SUG, GO_TO_URL};
use clap::error::ErrorKind;
use entities::{InternalError, PicaError};
use reqwest::{Error as ReqwestError, Response};
use serde::de::DeserializeOwned;

pub async fn handle_response<T>(
    req: Result<Response, ReqwestError>,
    printer: &Printer,
) -> Result<T, PicaError>
where
    T: DeserializeOwned,
{
    req.map_err(|e| {
        printer.stderr::<Pica>(
            &format!("{e}"),
            ErrorKind::Io,
            CHECK_INTERNET_CONNECTION_SUG,
            true,
        );
        InternalError::io_err(&format!("{e}"), None)
    })?
    .error_for_status()
    .map_err(|e| {
        printer.stderr::<Pica>(
            &format!("{e}"),
            ErrorKind::Io,
            CHECK_INTERNET_CONNECTION_SUG,
            true,
        );
        InternalError::io_err(&format!("{e}"), None)
    })?
    .json()
    .await
    .map_err(|e| {
        printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, CHECK_PARAMETERS_SUG, true);
        InternalError::io_err(&format!("{e}"), None)
    })
}

pub fn readline() -> Result<String, PicaError> {
    // TODO: If type is password then make it invisible
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| InternalError::io_err(&format!("{e}"), None))?;

    Ok(buffer)
}

pub fn open_browser(printer: &Printer, url: String) {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(url.as_str())
        .spawn()
        .and_then(|mut a| a.wait());

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer")
        .arg(url.as_str())
        .spawn()
        .and_then(|mut a| a.wait());

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(url.as_str())
        .spawn()
        .and_then(|mut a| a.wait());

    if result.is_err() {
        printer.stdout(&(GO_TO_URL.to_string() + url.as_str()));
    }
}
