use crate::api::BubbleApi;
use common::base64::Base64;
use common::http_types::{CheckMessages, Message, MessagesResponse, SendMessage};
use uuid::Uuid;

impl BubbleApi {
    pub async fn send_message(
        &self,
        client_uuids: Vec<Uuid>,
        message: Vec<u8>,
        group_uuid: Uuid,
    ) -> Result<(), reqwest::Error> {
        let message = Message {
            group_id: group_uuid,
            message: Base64(message),
        };
        let message = SendMessage {
            client_uuids,
            message,
        };
        self.client
            .post(&format!("{}/v1/messages", self.domain))
            .json(&message)
            .send()
            .await?;
        Ok(())
    }

    pub async fn receive_messages(
        &self,
        client_uuid: Uuid,
    ) -> Result<Vec<Message>, reqwest::Error> {
        let response: MessagesResponse = self
            .client
            .get(&format!("{}/v1/messages", self.domain))
            .json(&CheckMessages { client_uuid })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .unwrap();
        Ok(response.messages)
    }
}
