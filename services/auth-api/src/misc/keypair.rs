use anyhow;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde_json::json;
use std::fs;

pub struct KeyPair {
    pub private_key: String,
    pub public_key: String,
}

pub fn generate_key_pair() -> anyhow::Result<KeyPair> {
    println!("Generating RSA key pair...");

    // Generate a new RSA private key (2048 bits)
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = RsaPublicKey::from(&private_key);

    // Convert to PEM format
    let private_key_pem = private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
    let public_key_pem = public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;

    // Create the keys.json structure
    let keys_data = json!({
        "private_key": private_key_pem.as_str(),
        "public_key": public_key_pem,
        "key_id": "exchange_api_key_1",
        "algorithm": "RS256",
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    // Save to keys.json
    let keys_path = "keys.json";
    fs::write(keys_path, serde_json::to_string_pretty(&keys_data)?)?;

    println!("✅ Keys generated successfully!");
    println!("📁 Private key saved to: {}", keys_path);

    // Also save public key separately for easy access
    let public_key_data = json!({
        "keys": [{
            "kty": "RSA",
            "kid": "exchange_api_key_1",
            "use": "sig",
            "alg": "RS256",
            "n": URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be()),
            "e": URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be())
        }]
    });

    let public_key_path = "public_keys.json";
    fs::write(
        public_key_path,
        serde_json::to_string_pretty(&public_key_data)?,
    )?;

    println!("📄 Public key saved to: {}", public_key_path);

    Ok(KeyPair {
        private_key: private_key_pem.to_string(),
        public_key: public_key_pem,
    })
}
