pub struct GCR {
    decode_mappings: [u8; 32], // Index by 5-bit value, store decoded nibble
    encode_mappings: [u8; 16], // Index by nibble 0..15, store 5-bit encoded value
}

impl GCR {
    /// Constructs a new `GCR` (Group Code Recording) instance with precomputed
    /// lookup tables for efficient encoding and decoding operations.
    ///
    /// The `GCR` struct uses two lookup tables:
    ///
    /// - `decode_mappings`: A table that maps 5-bit encoded values (keys)
    ///   to their decoded 4-bit values. This is used for decoding operations.
    ///   Values that are considered invalid are initialized to `0xFF`.
    /// - `encode_mappings`: A table that maps 4-bit decoded values into their
    ///   respective 5-bit encoded counterparts, which is used for encoding
    ///   operations.
    ///
    /// The mapping pairs are predefined and represent the 4-bit to 5-bit
    /// encoding scheme:
    ///
    /// ```plaintext
    /// (Encoded, Decoded)
    /// (01010, 0), (01011, 1), (10010, 2), (10011, 3),
    /// (01110, 4), (01111, 5), (10110, 6), (10111, 7),
    /// (01001, 8), (11001, 9), (11010, 10), (11011, 11),
    /// (01101, 12), (11101, 13), (11110, 14), (10101, 15)
    /// ```
    ///
    /// Each `(encoded, decoded)` mapping is used to populate the appropriate
    /// indices in the lookup tables. For example:
    /// - `decode_mappings[encoded] = decoded`
    /// - `encode_mappings[decoded] = encoded`
    ///
    /// # Returns
    ///
    /// Returns an instance of the `GCR` struct with initialized `decode_mappings`
    /// and `encode_mappings`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let gcr = GCR::new();
    /// assert_eq!(gcr.decode_mappings[0b01010], 0); // Decodes "01010" to 0
    /// assert_eq!(gcr.encode_mappings[0], 0b01010); // Encodes 0 to "01010"
    /// ```
    pub fn new() -> Self {
        // Pre-compute lookup tables as arrays for O(1) access
        let mut decode_mappings = [0xFF; 32]; // Initialize with invalid marker
        let mut encode_mappings = [0u8; 16];

        // Populate the lookup tables
        let mapping_pairs = [
            (0b01010, 0),
            (0b01011, 1),
            (0b10010, 2),
            (0b10011, 3),
            (0b01110, 4),
            (0b01111, 5),
            (0b10110, 6),
            (0b10111, 7),
            (0b01001, 8),
            (0b11001, 9),
            (0b11010, 10),
            (0b11011, 11),
            (0b01101, 12),
            (0b11101, 13),
            (0b11110, 14),
            (0b10101, 15),
        ];

        for (encoded, decoded) in mapping_pairs {
            decode_mappings[encoded as usize] = decoded;
            encode_mappings[decoded as usize] = encoded as u8;
        }
        GCR { decode_mappings, encode_mappings }
    }

    /// Decodes a 64-bit encoded value into a `Vec<u8>` representing the original byte sequence.
    ///
    /// This function interprets the provided encoded value as consisting of 8 5-bit "quintuples"
    /// and converts them into 4 bytes of data using the `decode_mappings` array of the struct.
    ///
    /// # Parameters
    /// - `encoded_value` (`u64`): The 64-bit value to decode, containing 8 5-bit encoded segments.
    ///
    /// # Returns
    /// - `Option<Vec<u8>>`: A `Vec<u8>` containing the decoded bytes if the input is valid, or `None`
    ///   if any of the quintuples are invalid (i.e., mapped to `0xFF` in the `decode_mappings`).
    ///
    /// # Details
    /// - The function operates on 40 bits of input (8 quintuples of 5 bits each).
    /// - For each quintuple:
    ///   - It calculates the appropriate shift to extract the quintuple from the `encoded_value`.
    ///   - It uses the `decode_mappings` array for direct lookup to map the quintuple to a 4-bit value.
    /// - Decoding alternates between filling the high nibble and low nibble of a byte:
    ///   - If the nibble is the high nibble, it gets shifted left and stored.
    ///   - If the nibble is the low nibble, it gets combined with the high nibble to form a complete byte, which is appended to the result.
    /// - If any quintuple decodes to `0xFF`, the function returns `None` (indicating an invalid encoding).
    ///
    /// # Memory Management
    /// - The `Vec<u8>` result is pre-allocated with a capacity of 4 to match the exact number of decoded bytes.
    ///
    /// # Example
    /// ```rust
    /// let decoder = MyDecoderStruct {
    ///     decode_mappings: [ /* array mapping 32 possible quintuples to decoded nibbles */ ],
    /// };
    /// let encoded_value = 0x1A2B3C4D5E; // Some encoded value
    /// let decoded = decoder.decode_quintuple_new(encoded_value);
    ///
    /// match decoded {
    ///     Some(bytes) => println!("Decoded bytes: {:?}", bytes),
    ///     None => println!("Invalid encoding"),
    /// }
    /// ```
    fn decode_quintuple_new(&self, encoded_value: u64) -> Option<Vec<u8>> {
        let mut result = Vec::with_capacity(4); // Pre-allocate exact capacity
        let mut current_byte = 0u8;
        let mut is_high_nibble = true; // Start with high nibble for correct order

        // Process 8 quintuples (40 bits total)
        for j in 0..8 {
            let shift_amount = 35 - j * 5; // Calculate shift for each quintuple
            let quintuple_bits = ((encoded_value >> shift_amount) & 0x1f) as usize;

            // Direct array lookup instead of HashMap
            let decoded_nibble = self.decode_mappings[quintuple_bits];

            // Skip invalid encodings
            if decoded_nibble == 0xFF {
                return None;
            }

            if is_high_nibble {
                current_byte = decoded_nibble << 4;
            } else {
                current_byte |= decoded_nibble;
                result.push(current_byte);
                current_byte = 0;
            }
            is_high_nibble = !is_high_nibble;
        }

        Some(result)
    }

    /// Decodes the provided input byte slice (`value`) into a `Vec<u8>`.
    ///
    /// The `decode` method processes the input in chunks of 5 bytes, converting each chunk into a
    /// 64-bit integer by padding it with three leading zero bytes. It then calls the
    /// `decode_quintuple_new` method to decode the chunk into a vector of bytes.
    ///
    /// # Parameters
    /// - `value`: A reference to a slice of bytes (`&[u8]`) that represents the encoded input data.
    ///            This slice must have a length that is a multiple of 5 for full decoding.
    ///
    /// # Return
    /// - Returns `Some(Vec<u8>)` if decoding is successful for all chunks.
    /// - Returns `None` if any chunk fails to decode.
    ///
    /// # Example
    /// ```
    /// let mut decoder = MyDecoder::new();
    /// let input = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
    /// if let Some(decoded) = decoder.decode(&input) {
    ///     println!("Decoded output: {:?}", decoded);
    /// } else {
    ///     eprintln!("Failed to decode input.");
    /// }
    /// ```
    ///
    /// # Notes
    /// - This method uses `chunks_exact(5)` to divide the input into fixed-size chunks of 5.
    /// - For each chunk, a `u64` is constructed by appending three leading zero bytes to the 5-byte chunk to
    ///   match the byte size of a `u64`.
    /// - It relies on the `decode_quintuple_new` method to handle the actual decoding logic for
    ///   each chunk of reconstructed data. If `decode_quintuple_new` returns `None` for any chunk,
    ///   the entire decoding fails and the method returns `None`.
    ///
    /// # Panics
    /// This method does not panic under normal operation. However, improper implementation of
    /// `decode_quintuple_new` or incorrect input may result in unexpected behavior.
    pub fn decode(&mut self, value: &[u8]) -> Option<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        // Process chunks more efficiently using exact_chunks
        for chunk in value.chunks_exact(5) {
            let final_value = u64::from_be_bytes([
                0, 0, 0, // pad with zeros for the upper 3 bytes
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4],
            ]);

            if let Some(res) = self.decode_quintuple_new(final_value) {
                //println!("{:x?}", res);
                result.extend(res);
            } else {
                return None;
            }
        }
        Some(result)
    }

    /// Encodes a slice of bytes using a custom encoding scheme and returns the encoded data as a `Vec<u8>`.
    ///
    /// This method processes the input data in chunks of 4 bytes at a time. For each chunk:
    /// - Each byte is split into two 4-bit nibbles (high and low).
    /// - These nibbles are then mapped to corresponding 5-bit encoded values using a predefined `encode_mappings` table.
    /// - The 8 encoded nibbles (now 5-bit codes) are packed into a 40-bit value in a big-endian format.
    /// - Finally, the 40-bit value is split into 5 bytes and appended to the output vector.
    ///
    /// # Parameters
    /// - `value`: A slice of bytes (`&[u8]`) to be encoded.
    ///
    /// # Returns
    /// - `Vec<u8>`: A vector containing the encoded bytes.
    ///
    /// # Panics
    /// This function assumes that `self.encode_mappings` is properly defined (with valid mappings for all 4-bit values [0-15])
    /// and does not perform boundary checks on its size. Providing an invalid or incorrectly sized mapping may result in undefined behavior.
    ///
    /// # Example
    /// ```rust
    /// struct Encoder {
    ///     encode_mappings: [u8; 16],
    /// }
    ///
    /// let encoder = Encoder {
    ///     encode_mappings: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    /// };
    /// let input = vec![0x12, 0x34, 0x56, 0x78];
    /// let encoded = encoder.encode(&input);
    /// println!("{:?}", encoded);
    /// ```
    ///
    /// # Notes
    /// - The function processes the input in chunks of exactly 4 bytes. If the length of the input slice is not a multiple
    ///   of 4, the remaining bytes will be ignored. It is the caller's responsibility to handle padding or provide properly-sized input.
    pub fn encode(&self, value: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for chunk in value.chunks_exact(4) {
            // Prepare the 8 nibbles in the required order: high, low for each byte
            let nibbles = [
                chunk[0] >> 4,
                chunk[0] & 0x0F,
                chunk[1] >> 4,
                chunk[1] & 0x0F,
                chunk[2] >> 4,
                chunk[2] & 0x0F,
                chunk[3] >> 4,
                chunk[3] & 0x0F,
            ];

            // Pack 8 quintuples (5-bit codes) into a 40-bit big-endian value
            let mut acc: u64 = 0;
            for (j, &nib) in nibbles.iter().enumerate() {
                let code = self.encode_mappings[nib as usize] as u64;
                let shift_amount = 35 - (j as u32) * 5;
                acc |= code << shift_amount;
            }

            // Emit 5 bytes big-endian
            result.push(((acc >> 32) & 0xFF) as u8);
            result.push(((acc >> 24) & 0xFF) as u8);
            result.push(((acc >> 16) & 0xFF) as u8);
            result.push(((acc >> 8) & 0xFF) as u8);
            result.push((acc & 0xFF) as u8);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_works() {
        let mut gcr = GCR::new();
        let final_data: Vec<u8> = vec![0x52, 0x54, 0xb5, 0x29, 0x4b, 0x9a, 0xa6, 0xa5, 0x29, 0x4a];
        assert_eq!(
            gcr.decode(&final_data).unwrap(),
            vec![0x08, 0x01, 0x00, 0x01, 0x30, 0x30, 0x00, 0x00]
        );
    }

    #[test]
    fn encode_works() {
        let flux = GCR::new();
        let data: Vec<u8> = vec![0x08, 0x01, 0x00, 0x01, 0x30, 0x30, 0x00, 0x00];
        assert_eq!(
            flux.encode(&data),
            vec![0x52, 0x54, 0xb5, 0x29, 0x4b, 0x9a, 0xa6, 0xa5, 0x29, 0x4a]
        );
    }
}
