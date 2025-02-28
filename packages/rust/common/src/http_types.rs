use crate::base64::Base64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct CreateClient {
    pub signing_key: Base64,
    pub signature: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateClientResponse {
    pub client_uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateClient {
    pub signing_key: Base64,
    pub signature: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceKeyPackages {
    pub key_packages: Vec<Base64>,
}

#[derive(Serialize, Deserialize)]
pub struct KeyPackagePublic {
    pub key_package: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
    pub client_uuids: Vec<Uuid>,
    pub message: Message,
}

#[derive(Serialize, Deserialize)]
pub struct CheckMessages {
    pub client_uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct DeliveredMessage {
    pub message: Base64,
    pub received_date: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MessagesResponse {
    pub messages: Vec<DeliveredMessage>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub identity: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user_uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct ConfirmEmail {
    pub token: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct SessionTokenRequest {
    pub token: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct SessionTokenResponse {
    pub user_uuid: Uuid,
    pub bearer: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForgotEmail {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordReset {
    pub password: String,
    pub token: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordResetCheck {
    pub token: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeEmail {
    pub new_email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteUser {
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateIdentity {
    pub identity: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct PublicUser {
    pub uuid: Uuid,
    pub username: String,
    pub name: String,
    pub primary_client_uuid: Option<Uuid>,
    pub identity: Base64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicClient {
    pub user_uuid: Uuid,
    pub uuid: Uuid,
    pub signing_key: Base64,
    pub signature: Base64,
}

#[derive(Serialize, Deserialize)]
pub struct ClientsResponse {
    pub clients: Vec<PublicClient>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisteredClientsResponse {
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub primary_client_uuid: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct Search {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub users: Vec<PublicUser>,
}
