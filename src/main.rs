mod config;

use std::str::from_utf8;

use actix_web::{
    error, get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder, Result,
};
// use alloy_primitives::U256;
use config::Config;
// use ethers::contract::{abigen, Contract, ContractCall};
use ethers::{
    abi::{encode, token::Token},
    providers::{Http, Provider},
    types::{Address, Bytes, TransactionRequest, U256},
};
use lazy_static::lazy_static;
use serde::Serialize;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;
const TRANSPORT: &str = "http";

lazy_static! {
    static ref CONFIG: Config = Config::from_env();
    static ref PROVIDER: Provider<Http> =
        Provider::<Http>::try_from(CONFIG.l1_rpc_http.as_str()).expect("failed to create provider");
}

fn api_path(path: &str) -> String {
    let mut path = path.to_string();
    if path.starts_with("/") {
        path.remove(0);
    }
    format!("{}://{}:{}/{}", TRANSPORT, HOST, PORT, path)
}

#[derive(Serialize)]
struct ERC721Metadata {
    name: String,
    description: String,
    image: String,
}

#[get("/svg/{nfteeAddress}/{tokenId}")]
async fn nft_image(path: web::Path<(Address, String)>) -> Result<impl Responder> {
    let (nftee_address, token_id) = path.into_inner();
    let token_id = U256::from_dec_str(&token_id).map_err(error::ErrorBadRequest)?;

    // call `tokenURI(uint256 tokenId)` to get image data
    let sig = "0xc87b56dd".parse::<Bytes>().unwrap();
    let args: Bytes = encode(&[Token::Uint(token_id)]).into();
    let calldata = [sig, args].concat();
    let tx = TransactionRequest::new().data(calldata).to(nftee_address);
    let token_data = PROVIDER
        .call_raw(&tx.into())
        .await
        .map_err(error::ErrorExpectationFailed)?;
    let uft8_data = from_utf8(&token_data)?;
    let utf8_lines: Vec<String> = uft8_data
        .replace("\0", "")
        .replace("", "")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .split('\n')
        .map(|l| l.to_owned())
        .collect();

    // pack image data into SVG
    let nft_content = utf8_lines
        .into_iter()
        .map(|line| format!(r###"<tspan x="0" dy="1.2em">{}</tspan>"###, line))
        .collect::<String>();
    let image_data = format!(
        r###"<?xml version="1.0" standalone="yes"?>
<svg height="250px" width="250px" xmlns="http://www.w3.org/2000/svg" version="1.1">
<text x="0" y="15" font-family="monospace" fill="green" xml:space="preserve">{}</text>
</svg>"###,
        nft_content
    );

    // Ok(web::Bytes::from(image_data))
    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(image_data))
}

/// Calling `tokenURI(tokenId)` on NFTEE contract gets this URI.
/// Returns metadata in JSON format.
#[get("/metadata/{nfteeAddress}/{tokenId}")]
async fn nft_data(path: web::Path<(Address, String)>) -> Result<impl Responder> {
    let (nftee_address, token_id) = path.into_inner();
    let token_id = U256::from_dec_str(&token_id).map_err(error::ErrorBadRequest)?;

    // return metadata in JSON format
    let metadata = ERC721Metadata {
        name: token_id.to_string(),
        description: "NFT rendered from ASCII data.".to_string(),
        image: api_path(&format!("/svg/{:#?}/{}", nftee_address, token_id)),
    };

    Ok(web::Json(metadata))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(nft_data).service(nft_image))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
