use anyhow::Result;
use crate::led::data::{LedData, LedUpdate};
use log::*;
use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};
use esp_idf_svc::{
    http::server::EspHttpServer,
};

static INDEX_HTML: &str = include_str!("html/http_server_page.html");
static FORM_HTML: &str = include_str!("html/form.html");

// Max payload length
const MAX_LEN: usize = 8192;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

fn create_server() -> Result<EspHttpServer<'static>> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    Ok(EspHttpServer::new(&server_configuration)?)
}

pub fn http_routes(tx: std::sync::mpsc::SyncSender<LedUpdate>) -> Result<()> {
    let mut server = create_server()?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write(INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler("/form", Method::Get, |req| {
        req.into_ok_response()?
            .write(FORM_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler::<anyhow::Error, _>("/led", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        match serde_json::from_slice::<LedData>(&buf) {
            Ok(form) => {
                info!("Recieved Post Request");
                write!(resp, "Received Led Data")?;
                match LedUpdate::from_led_data(form) {
                    Ok(converted) => {
                        match tx.try_send(converted) {
                            Ok(_) => {
                                write!(resp, "Success")?;
                            }
                            Err(e) => {
                                resp.write_all(e.to_string().as_bytes())?;
                            }
                        }
                    }
                    Err(e) => {
                        resp.write_all(e.to_string().as_bytes())?;
                    }
                }
            }
            Err(e) => {
                resp.write_all(e.to_string().as_bytes())?;
            }
        }

        Ok(())
    })?;

    core::mem::forget(server);
    Ok(())
}
