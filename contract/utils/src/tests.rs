/*
 * Tests for the contract.
 */
#[cfg(test)]
mod tests {
    use crate::{ecrecover::recover, Gateway};

    use rand_core::OsRng;

    use k256::{
        ecdsa::{
            recoverable,
            signature::{Signature, Signer},
            SigningKey,
        },
        elliptic_curve::rand_core,
    };

    #[test]
    fn test_recover() {
        let signing_key: SigningKey = k256::ecdsa::SigningKey::random(&mut OsRng);

        let verifying_key = signing_key.verifying_key();

        // Message to sign
        let message = b"Hello";

        // Signature after signing
        let signature: recoverable::Signature = signing_key.sign(message);

        // Recover the public key from the signature
        let recovered_key = recover(message, signature.as_bytes());

        // Verify the signature
        assert_eq!(&verifying_key, &recovered_key);
    }

    #[test]
    fn test_sign_msg() {
        let contract = Gateway::default();

        let hash = contract.sign_message("Hello".to_string());

        assert_eq!(
            hash,
            "0xc39b73d55958ab15b2b6f0efad7eb0ee661256df12c47bf9e1fe819026ec7c48"
        );
    }

    #[test]
    fn test_decode() {
        let contract = Gateway::default();

        let payload = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000".to_string();
        // Remove 0x from the beginning of the string
        let clean_payload = &payload[2..payload.len()];

        let result = contract.abi_decode(clean_payload.to_string());

        assert_eq!(result[0], "hello");
    }

    #[test]
    fn test_encode() {
        let contract = Gateway::default();
        let message = "hello".to_string();
        let result = contract.abi_encode(message);

        let expected_payload = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000".to_string();
        // Remove 0x from the beginning of the string
        let clean_payload = &expected_payload[2..expected_payload.len()];

        assert_eq!(result, clean_payload);
    }
}
