use starknet_crypto::FieldElement;

/// Parse bytes from hex string
pub fn parse_bytes(arg: &str) -> Result<Vec<u8>, String> {
    hex::decode(arg).map_err(|e| format!("Failed to parse bytes: {}", e))
}

/// Parse FieldElement from hex string
pub fn parse_field_element(arg: &str) -> Result<FieldElement, String> {
    hex::decode(arg)
        .map(|bytes| FieldElement::from_byte_slice_be(bytes.as_slice()).unwrap())
        .map_err(|e| format!("Failed to parse bytes: {}", e))
}
