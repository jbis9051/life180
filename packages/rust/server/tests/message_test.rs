use crate::crypto_helper::{generate_ed25519_keypair, PRIVATE, PUBLIC};
use crate::helper::{start_server, TempDatabase};
use axum::http::StatusCode;
use common::base64::Base64;
use common::http_types::{CheckMessages, CreateUser, MessagesResponse, SendMessage};
use uuid::Uuid;

mod crypto_helper;
mod helper;

#[tokio::test]
async fn test_single_message() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "test_username".to_string(),
        password: "test_password".to_string(),
        name: "test_name".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);
    let (_, client_uuid) = helper::create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    let request_messages = CheckMessages {
        client_uuid: client_uuid.to_string(),
    };
    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.json::<MessagesResponse>().await.messages.len(), 0);
    let testmessage1 = "test message";
    let message = SendMessage {
        client_uuids: vec![client_uuid.to_string()],
        message: Base64(testmessage1.as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    let messages = res.json::<MessagesResponse>().await.messages;
    assert_eq!(messages.len(), 1);
    assert_eq!(testmessage1.as_bytes().to_vec(), messages[0].0);
}

#[tokio::test]
async fn test_multiple_messages() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;
    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "test_username".to_string(),
        password: "test_password".to_string(),
        name: "test_name".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);
    let (_, client_uuid) = helper::create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    let message_1 = SendMessage {
        client_uuids: vec![client_uuid.to_string()],
        message: Base64("test message 1".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message_1).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let message_2 = SendMessage {
        client_uuids: vec![client_uuid.to_string()],
        message: Base64("test message 2".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message_2).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let message_3 = SendMessage {
        client_uuids: vec![client_uuid.to_string()],
        message: Base64("test message 3".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message_3).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let request_messages = CheckMessages {
        client_uuid: client_uuid.to_string(),
    };
    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    let messages = res.json::<MessagesResponse>().await.messages;
    assert_eq!(messages.len(), 3);
    assert_eq!("test message 1".as_bytes().to_vec(), messages[0].0);
    assert_eq!("test message 2".as_bytes().to_vec(), messages[1].0);
    assert_eq!("test message 3".as_bytes().to_vec(), messages[2].0);
}

#[tokio::test]
async fn test_invalid_uuid() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "test_username".to_string(),
        password: "test_password".to_string(),
        name: "test_name".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);
    let (_, client_uuid) = helper::create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    let bad_uuid = Uuid::new_v4().to_string();

    let request_messages = CheckMessages {
        client_uuid: bad_uuid.clone(),
    };
    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let request_messages = CheckMessages {
        client_uuid: "bad uuid".to_string(),
    };
    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let message = SendMessage {
        client_uuids: vec![bad_uuid.clone()],
        message: Base64("test message".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let message = SendMessage {
        client_uuids: vec![client_uuid.to_string(), bad_uuid],
        message: Base64("test message".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let message = SendMessage {
        client_uuids: vec!["bad uuid".to_string()],
        message: Base64("test message".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let message = SendMessage {
        client_uuids: vec![],
        message: Base64("test message".as_bytes().to_vec()),
    };
    let res = client
        .post("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message).unwrap())
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bad_user() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let test_keypair = generate_ed25519_keypair();

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "test_username".to_string(),
        password: "test_password".to_string(),
        name: "test_name".to_string(),
        identity: Base64(test_keypair.public.to_bytes().to_vec()),
    };
    let (token, _) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);
    let (_, client_uuid) = helper::create_client(
        &test_keypair.public.to_bytes(),
        &test_keypair.secret.to_bytes(),
        &bearer,
        &client,
    )
    .await;

    let bad_keypair = generate_ed25519_keypair();
    let bad_user = CreateUser {
        email: "bad@gmail.com".to_string(),
        username: "bad_username".to_string(),
        password: "bad_password".to_string(),
        name: "bad_name".to_string(),
        identity: Base64(bad_keypair.public.to_bytes().to_vec()),
    };
    let (bad_token, _) = helper::initialize_user(db.pool(), &client, &bad_user)
        .await
        .unwrap();

    let bad_bearer = format!("Bearer {}", bad_token);
    let (_, _bad_client_uuid) = helper::create_client(
        &bad_keypair.public.to_bytes(),
        &bad_keypair.secret.to_bytes(),
        &bad_bearer,
        &client,
    )
    .await;

    let request_messages = CheckMessages {
        client_uuid: client_uuid.to_string(),
    };
    let res = client
        .get("/message")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_messages).unwrap())
        .header("Authorization", bad_bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
