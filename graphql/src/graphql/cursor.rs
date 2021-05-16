pub trait Cursor: Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn encode(&self) -> String {
        base64::encode_config(
            serde_json::to_string(self).expect("Serialize Cursor failed"),
            base64::URL_SAFE,
        )
    }

    fn decode(v: &str) -> Option<Self> {
        base64::decode_config(v, base64::URL_SAFE)
            .ok()
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|v| serde_json::from_str(&v).ok())
    }
}
