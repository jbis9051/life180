use crate::helper::{create_client, start_server, TempDatabase};
use axum::http::StatusCode;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use sqlx::Row;

use uuid::Uuid;

use crate::crypto_helper::{PRIVATE, PUBLIC};
use common::base64::Base64;
use common::http_types::{
    ClientsResponse, CreateClient, CreateClientResponse, CreateUser, KeyPackagePublic,
    PublicClient, ReplaceKeyPackages,
};
use server::types::{CIPHERSUITES, SIGNATURE_SCHEME};

mod crypto_helper;
mod helper;

#[tokio::test]
async fn test_client_crud() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    // Ensure there are no clients
    let res = client
        .get(&format!("/v1/user/{}/clients", user.uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let payload: ClientsResponse = res.json().await;

    assert_eq!(payload.clients.len(), 0);

    // Create a Client
    let signature_keypair = SignatureKeyPair::new(SIGNATURE_SCHEME).unwrap();

    let user_keypair = Keypair {
        public: PublicKey::from_bytes(PUBLIC).unwrap(),
        secret: SecretKey::from_bytes(PRIVATE).unwrap(),
    };

    let signature_of_signing_key = user_keypair.sign(signature_keypair.public());

    let create_client = CreateClient {
        signing_key: Base64(signature_keypair.public().to_vec()),
        signature: Base64(signature_of_signing_key.to_bytes().to_vec()),
    };

    let res = client
        .post("/v1/client")
        .header("Authorization", bearer.clone())
        .json(&create_client)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res: CreateClientResponse = res.json().await;

    let client_uuid = res.client_uuid;

    // Ensure the client is created
    let res = client
        .get(&format!("/v1/user/{}/clients", user.uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let payload: ClientsResponse = res.json().await;

    assert_eq!(payload.clients.len(), 1);
    assert_eq!(payload.clients[0].user_uuid, user.uuid);
    assert_eq!(payload.clients[0].uuid, client_uuid);
    assert_eq!(payload.clients[0].signing_key.0, signature_keypair.public());
    assert_eq!(
        payload.clients[0].signature.0,
        &signature_of_signing_key.to_bytes()
    );

    // Check that the client can be retrieved

    let res = client
        .get(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let payload: PublicClient = res.json().await;

    assert_eq!(payload.user_uuid, user.uuid);
    assert_eq!(payload.uuid, client_uuid);
    assert_eq!(payload.signing_key.0, signature_keypair.public());
    assert_eq!(payload.signature.0, &signature_of_signing_key.to_bytes());

    // Update the Client with a new signing key

    let signature_keypair = SignatureKeyPair::new(SIGNATURE_SCHEME).unwrap();

    let signature_of_signing_key = user_keypair.sign(signature_keypair.public());

    let create_client = CreateClient {
        signing_key: Base64(signature_keypair.public().to_vec()),
        signature: Base64(signature_of_signing_key.to_bytes().to_vec()),
    };

    let res = client
        .patch(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .json(&create_client)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    // Ensure the client is updated

    let res = client
        .get(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let payload: PublicClient = res.json().await;

    assert_eq!(payload.user_uuid, user.uuid);
    assert_eq!(payload.uuid, client_uuid);
    assert_eq!(payload.signing_key.0, signature_keypair.public());
    assert_eq!(payload.signature.0, &signature_of_signing_key.to_bytes());

    // Delete the Client

    let res = client
        .delete(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    // Ensure the client is deleted

    let res = client
        .get(&format!("/v1/user/{}/clients", user.uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let payload: ClientsResponse = res.json().await;

    assert_eq!(payload.clients.len(), 0);

    // Ensure the client cannot be retrieved

    let res = client
        .get(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_key_packages() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    // Create a Client
    let backend = &OpenMlsRustCrypto::default();

    let (signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // Ensure there are no key packages

    let res = client
        .get(&format!("/v1/client/{}/key_package", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Upload Key Packages

    let identity = format!("client_{}_{}", user.uuid, client_uuid);
    let credential = Credential::new(identity.into_bytes(), CredentialType::Basic).unwrap();

    let mut key_packages = Vec::new();

    for _ in 0..5 {
        let key_package = KeyPackage::builder()
            .build(
                CryptoConfig {
                    ciphersuite: CIPHERSUITES,
                    version: ProtocolVersion::default(),
                },
                backend,
                &signature_keypair,
                CredentialWithKey {
                    credential: credential.clone(),
                    signature_key: SignaturePublicKey::from(signature_keypair.public()),
                },
            )
            .unwrap();
        key_packages.push(Base64(key_package.tls_serialize_detached().unwrap()));
    }

    let payload = ReplaceKeyPackages { key_packages };

    let res = client
        .post(&format!("/v1/client/{}/key_packages", client_uuid))
        .header("Authorization", bearer.clone())
        .json(&payload)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    // Get a Key Package

    let res = client
        .get(&format!("/v1/client/{}/key_package", client_uuid))
        .header("Authorization", bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let _: KeyPackagePublic = res.json().await;

    let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM key_package;")
        .fetch_one(db.pool())
        .await
        .unwrap()
        .get("count");

    assert_eq!(count, 4); // ensure that one key package is deleted
}

// negative tests

#[tokio::test]
async fn test_create_client_bad_signature() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let signature_keypair = SignatureKeyPair::new(SIGNATURE_SCHEME).unwrap();

    let user_keypair = Keypair {
        public: PublicKey::from_bytes(PUBLIC).unwrap(),
        secret: SecretKey::from_bytes(PRIVATE).unwrap(),
    };

    let signature_of_signing_key = user_keypair.sign(signature_keypair.public());

    let create_client = CreateClient {
        signing_key: Base64(signature_keypair.public().to_vec()),
        signature: Base64(vec![0; signature_of_signing_key.to_bytes().len()]),
    };

    let res = client
        .post("/v1/client")
        .header("Authorization", bearer.clone())
        .json(&create_client)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_client_bad_auth() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let (_signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // create a second user

    let bad_user = CreateUser {
        email: "bad@gmail.com".to_string(),
        username: "badusername".to_string(),
        password: "badpassword".to_string(),
        name: "badname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };

    let (bad_token, _bad_user) = helper::initialize_user(db.pool(), &client, &bad_user)
        .await
        .unwrap();

    let bad_bearer = format!("Bearer {}", bad_token);

    // try to update the client with the second user's token

    let signature_keypair = SignatureKeyPair::new(SIGNATURE_SCHEME).unwrap();

    let user_keypair = Keypair {
        public: PublicKey::from_bytes(PUBLIC).unwrap(),
        secret: SecretKey::from_bytes(PRIVATE).unwrap(),
    };
    let signature_of_signing_key = user_keypair.sign(signature_keypair.public());

    let create_client = CreateClient {
        signing_key: Base64(signature_keypair.public().to_vec()),
        signature: Base64(signature_of_signing_key.to_bytes().to_vec()),
    };

    let res = client
        .patch(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bad_bearer.clone())
        .json(&create_client)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_client_bad_signature() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let (_signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // try to update the client with a bad signature

    let _backend = &OpenMlsRustCrypto::default();
    let signature_keypair = SignatureKeyPair::new(SIGNATURE_SCHEME).unwrap();

    let user_keypair = Keypair {
        public: PublicKey::from_bytes(PUBLIC).unwrap(),
        secret: SecretKey::from_bytes(PRIVATE).unwrap(),
    };
    let signature_of_signing_key = user_keypair.sign(signature_keypair.public());

    let create_client = CreateClient {
        signing_key: Base64(signature_keypair.public().to_vec()),
        signature: Base64(vec![0; signature_of_signing_key.to_bytes().len()]),
    };

    let res = client
        .patch(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bearer.clone())
        .json(&create_client)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_client_bad_auth() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, _user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let (_signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // create a second user

    let bad_user = CreateUser {
        email: "bad@gmail.com".to_string(),
        username: "badusername".to_string(),
        password: "badpassword".to_string(),
        name: "badname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };

    let (bad_token, _bad_user) = helper::initialize_user(db.pool(), &client, &bad_user)
        .await
        .unwrap();

    let bad_bearer = format!("Bearer {}", bad_token);

    // try to delete the client with the second user's token

    let res = client
        .delete(&format!("/v1/client/{}", client_uuid))
        .header("Authorization", bad_bearer.clone())
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_replace_key_packages_bad_auth() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let (signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // create a second user

    let bad_user = CreateUser {
        email: "bad@gmail.com".to_string(),
        username: "badusername".to_string(),
        password: "badpassword".to_string(),
        name: "badname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };

    let (bad_token, _bad_user) = helper::initialize_user(db.pool(), &client, &bad_user)
        .await
        .unwrap();

    let bad_bearer = format!("Bearer {}", bad_token);

    // try to upload a key package with the second user's token

    let identity = format!("client_{}_{}", user.uuid, client_uuid);
    let credential = Credential::new(identity.into_bytes(), CredentialType::Basic).unwrap();

    let mut key_packages = Vec::new();

    let backend = &OpenMlsRustCrypto::default();

    let key_package = KeyPackage::builder()
        .build(
            CryptoConfig {
                ciphersuite: CIPHERSUITES,
                version: ProtocolVersion::default(),
            },
            backend,
            &signature_keypair,
            CredentialWithKey {
                credential: credential.clone(),
                signature_key: SignaturePublicKey::from(signature_keypair.public()),
            },
        )
        .unwrap();
    key_packages.push(Base64(key_package.tls_serialize_detached().unwrap()));

    let payload = ReplaceKeyPackages { key_packages };

    let res = client
        .post(&format!("/v1/client/{}/key_packages", client_uuid))
        .header("Authorization", bad_bearer.clone())
        .json(&payload)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_replace_key_packages_id() {
    let db = TempDatabase::new().await;
    let client = start_server(db.pool().clone()).await;

    let created_user = CreateUser {
        email: "test@gmail.com".to_string(),
        username: "testusername".to_string(),
        password: "testpassword".to_string(),
        name: "testname".to_string(),
        identity: Base64(PUBLIC.to_vec()),
    };
    let (token, user) = helper::initialize_user(db.pool(), &client, &created_user)
        .await
        .unwrap();

    let bearer = format!("Bearer {}", token);

    let (signature_keypair, client_uuid) = create_client(PUBLIC, PRIVATE, &bearer, &client).await;

    // try to upload a key package a bad identity

    let identity = format!("client_{}_{}", user.uuid, Uuid::new_v4());
    let credential = Credential::new(identity.into_bytes(), CredentialType::Basic).unwrap();

    let mut key_packages = Vec::new();

    let backend = &OpenMlsRustCrypto::default();

    let key_package = KeyPackage::builder()
        .build(
            CryptoConfig {
                ciphersuite: CIPHERSUITES,
                version: ProtocolVersion::default(),
            },
            backend,
            &signature_keypair,
            CredentialWithKey {
                credential: credential.clone(),
                signature_key: SignaturePublicKey::from(signature_keypair.public()),
            },
        )
        .unwrap();
    key_packages.push(Base64(key_package.tls_serialize_detached().unwrap()));

    let payload = ReplaceKeyPackages { key_packages };

    let res = client
        .post(&format!("/v1/client/{}/key_packages", client_uuid))
        .header("Authorization", bearer.clone())
        .json(&payload)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
