use paradocs::*;

#[tokio::test]
async fn test_crate_root() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_module() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::fs").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_struct() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::net::TcpStream").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_trait() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::io::AsyncReadExt").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_enum() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::io::ErrorKind").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_function() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::spawn").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_macro() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::task_local").await.unwrap();
    eprintln!("{:#?}", document);
}

#[tokio::test]
async fn test_attribute() {
    let client = DocsClient::default();
    let document = client.get_document("tokio", "tokio::main").await.unwrap();
    eprintln!("{:#?}", document);
}
