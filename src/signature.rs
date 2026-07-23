//! Webhook signature helpers (HMAC-SHA256).
//!
//! Webhooks are how operators tell us "this payment succeeded". Without a
//! signature check, anyone who guesses your URL could POST a fake "success"
//! and get goods for free. Always verify.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute the hex HMAC-SHA256 of `payload` with `secret`.
pub fn sign_hmac_sha256(payload: &[u8], secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

/// Constant-time verify of a hex signature against `payload`.
pub fn verify_hmac_sha256(payload: &[u8], signature_hex: &str, secret: &[u8]) -> bool {
    let expected = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(payload);
    // `verify_slice` is constant-time — resists timing attacks.
    mac.verify_slice(&expected).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_roundtrips() {
        let secret = b"top-secret";
        let body = br#"{"status":"success"}"#;
        let sig = sign_hmac_sha256(body, secret);
        assert!(verify_hmac_sha256(body, &sig, secret));
    }

    #[test]
    fn wrong_secret_fails() {
        let body = br#"{"status":"success"}"#;
        let sig = sign_hmac_sha256(body, b"right");
        assert!(!verify_hmac_sha256(body, &sig, b"wrong"));
    }

    #[test]
    fn tampered_body_fails() {
        let secret = b"top-secret";
        let sig = sign_hmac_sha256(br#"{"amount":100}"#, secret);
        assert!(!verify_hmac_sha256(br#"{"amount":999}"#, &sig, secret));
    }
}
