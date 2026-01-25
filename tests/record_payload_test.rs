use keyring_cli::crypto::{
    record::{decrypt_payload, encrypt_payload, RecordPayload},
    CryptoManager,
};

#[test]
fn record_payload_encrypt_decrypt() {
    let mut crypto = CryptoManager::new();
    crypto.initialize("master").unwrap();

    let payload = RecordPayload {
        name: "github".to_string(),
        username: Some("octo".to_string()),
        password: "secret".to_string(),
        url: Some("https://github.com".to_string()),
        notes: None,
        tags: vec!["work".to_string()],
    };

    let (cipher, nonce) = encrypt_payload(&crypto, &payload).unwrap();
    let decoded = decrypt_payload(&crypto, &cipher, &nonce).unwrap();
    assert_eq!(decoded.name, "github");
}
