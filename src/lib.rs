pub struct GCR {
    decode_mappings: [u8; 32], // Index by 5-bit value, store decoded nibble
    encode_mappings: [u8; 16], // Index by nibble 0..15, store 5-bit encoded value
}

const QUINTUPLE_SIZE: usize = 5;

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
        GCR {
            decode_mappings,
            encode_mappings,
        }
    }

    /// Decodes a 40-bit encoded value into a vector of bytes (maximum 4 bytes).
    ///
    /// This function processes an encoded 40-bit quintuple value, where each 5-bit segment (quintuple)
    /// translates to its corresponding decoded nibble using a precomputed `decode_mappings` array.
    /// The function decodes 8 quintuples (2 per byte) and returns a `Vec<u8>` containing the resulting bytes.
    ///
    /// If any quintuple cannot be decoded (i.e., its mapping results in `0xFF`, which is treated as invalid),
    /// the function returns `None`.
    ///
    /// ### Parameters
    /// - `encoded_value (u64)`: The 40-bit value to decode. It should be properly aligned so that the relevant bits
    ///   can be shifted and masked correctly during decoding.
    ///
    /// ### Returns
    /// - `Option<Vec<u8>>`: A `Some` containing the decoded vector of up to 4 bytes if decoding is successful,
    ///   or `None` if any quin-tuple is invalid.
    ///
    /// ### Precondition
    /// - The caller must ensure that the `self.decode_mappings` array is properly populated so that each 5-bit value
    ///   (0 through 31) either maps to a valid 4-bit nibble or `0xFF` for invalid encodings.
    ///
    /// ### Algorithm
    /// - For each pair of consecutive quintuples (2 quintuples per iteration):
    ///   1. Shift and mask the first quintuple from the encoded value.
    ///   2. Look up its corresponding nibble in `decode_mappings`.
    ///   3. Repeat for the second quintuple in the pair.
    ///   4. If either quintuple mapping results in an invalid value (`0xFF`), terminate early and return `None`.
    ///   5. Combine the two valid decoded nibbles into a single byte and append to the result.
    ///
    /// ### Example
    /// ```rust
    /// let decoder = MyDecoder::new();
    /// let encoded_value: u64 = 0b11110_00001_11110_00001_11110_00001_11110_00001; // Example encoded value
    /// let decoded = decoder.decode_quintuple(encoded_value);
    /// assert_eq!(decoded, Some(vec![0xF1, 0xF1, 0xF1, 0xF1])); // Decoding successful
    ///
    /// let invalid_encoded_value: u64 = 0b11110_11110_11110_11110_11110_11110_11110_11111; // Invalid encoding
    /// let decoded = decoder.decode_quintuple(invalid_encoded_value);
    /// assert_eq!(decoded, None); // Decoding failed due to an invalid quintuple
    /// ```
    ///
    /// ### Notes
    /// - The function uses a pre-allocated vector (`Vec`) with a capacity of 4 to maximize efficiency and prevent resizing.
    /// - The function assumes `QUINTUPLE_SIZE` is defined as a constant equal to 5 (5 bits per quintuple).
    /// - This function is particularly optimized for scenarios where the decoding process is executed frequently by utilizing
    ///   direct array lookups rather than more expensive structures like `HashMap`.
    fn decode_quintuple(&self, encoded_value: u64) -> Option<Vec<u8>> {
        let mut result = Vec::with_capacity(4); // Pre-allocate exact capacity

        // Process 8 quintuples (40 bits total)
        for j in (0..8).step_by(2) {
            // Direct array lookup instead of HashMap
            let decoded_nibble_high =
                self.decode_mappings[((encoded_value >> 35 - j * QUINTUPLE_SIZE) & 0x1f) as usize];
            // Direct array lookup instead of HashMap
            let decoded_nibble_low = self.decode_mappings
                [((encoded_value >> 35 - (j + 1) * QUINTUPLE_SIZE) & 0x1f) as usize];
            // Skip invalid encodings
            if decoded_nibble_high == 0xFF || decoded_nibble_low == 0xFF {
                return None;
            }

            result.push(decoded_nibble_high << 4 | decoded_nibble_low);
        }

        Some(result)
    }

    /// Decodes a slice of bytes using a specific decoding logic implemented in conjunction with the `decode_quintuple` method.
    ///
    /// This method processes the given input slice `value`, dividing it into fixed-size chunks (of size `QUINTUPLE_SIZE`),
    /// and applies decoding logic to each chunk. The decoded bytes are collected and returned as a `Vec<u8>`.
    ///
    /// # Parameters
    /// - `value`: A slice of bytes (`&[u8]`) that represents the encoded input to be decoded.
    ///
    /// # Returns
    /// - `Some(Vec<u8>)`: A `Vec<u8>` containing the decoded bytes, if decoding is successful.
    /// - `None`: Returned if decoding fails for any of the data chunks.
    ///
    /// # Methodology
    /// 1. The input slice `value` is iterated in fixed-size chunks. This is achieved using the `chunks_exact`
    ///    method, which ensures efficient processing of chunks of size `QUINTUPLE_SIZE`.
    /// 2. For each chunk, it is converted into a 64-bit integer by padding the upper 3 bytes with zeros.
    /// 3. The method `decode_quintuple` (presumably implemented elsewhere in the code) is invoked with the 64-bit integer.
    ///    - If `decode_quintuple` returns a valid result, the decoded data is appended to the result vector (`result`).
    ///    - If `decode_quintuple` fails for any chunk, the function returns `None`.
    /// 4. If all chunks are successfully decoded, the accumulated result is wrapped in `Some` and returned.
    ///
    /// # Example
    /// ```
    /// let decoder = MyDecoder::new(); // Assuming a struct that implements the method
    /// let encoded_data: &[u8] = &[/* encoded bytes */];
    /// if let Some(decoded_data) = decoder.decode(encoded_data) {
    ///     println!("Decoded data: {:?}", decoded_data);
    /// } else {
    ///     println!("Failed to decode the data.");
    /// }
    /// ```
    ///
    /// # Note
    /// The size of `QUINTUPLE_SIZE` and the implementation of the `decode_quintuple` method
    /// are critical for the proper functionality of this method. Ensure these are defined
    /// and implemented correctly in the same context.
    ///
    /// # Assumptions
    /// - The `QUINTUPLE_SIZE` constant is defined and is less than or equal to 5.
    /// - The `decode_quintuple` function is implemented to correctly decode a `u64` value into a `Vec<u8>`.
    pub fn decode(&self, value: &[u8]) -> Option<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        // Process chunks more efficiently using exact_chunks
        for chunk in value.chunks_exact(QUINTUPLE_SIZE) {
            let final_value = u64::from_be_bytes([
                0, 0, 0, // pad with zeros for the upper 3 bytes
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4],
            ]);

            if let Some(res) = self.decode_quintuple(final_value) {
                result.extend(res);
            } else {
                return None;
            }
        }
        Some(result)
    }

    /// Encodes a 4-byte sequence into a 40-bit number using predefined mappings.
    ///
    /// This function takes a reference to a slice of 4 bytes (`decoded_value`)
    /// and encodes it into a `u64` (64-bit unsigned integer) using a provided
    /// `encode_mappings` array. Each byte is split into two 4-bit halves, and
    /// each half is converted into an encoded value based on the mapping table.
    /// These encoded values are then combined into a single 64-bit value, with
    /// each encoded value taking up a specific bit range in the result.
    ///
    /// # Parameters
    /// - `decoded_value`: A reference to an array of 4 bytes to be encoded.
    ///   The slice must be exactly 4 bytes long, or the behavior is undefined.
    ///
    /// # Returns
    /// - A `u64` value representing the encoded result of the given slice.
    ///
    /// # Panics
    /// - This function will not panic under normal operation as long as the
    ///   `decoded_value` slice is exactly 4 bytes long and the indices used
    ///   for `encode_mappings` are within bounds.
    ///
    /// # Assumptions
    /// - `self.encode_mappings` is an array of values that maps 4-bit components
    ///   (0 through 15) to their corresponding encoded values.
    /// - The constant `QUINTUPLE_SIZE` determines the size of the bit shift
    ///   and should align with the encoding rules.
    ///
    /// # Implementation Details
    /// - Each byte in the `decoded_value` slice is divided into two 4-bit
    ///   components:
    ///   - The high nibble (upper 4 bits): `decoded_value[i] >> 4`
    ///   - The low nibble (lower 4 bits): `decoded_value[i] & 0x0F`
    /// - These components are looked up in `self.encode_mappings` to obtain
    ///   their encoded values.
    /// - The encoded values are right-shifted into their respective positions
    ///   within the 64-bit result (`acc`), based on their sequence order.
    ///
    /// # Example
    /// ```rust
    /// // Assuming `QUINTUPLE_SIZE` is defined and `self.encode_mappings` is
    /// // already initialized correctly:
    /// let decoded_data: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// let encoded_value = your_object.encode_quintuple(&decoded_data);
    /// println!("Encoded Value: {:#X}", encoded_value);
    /// ```
    ///
    /// # Output
    /// - The function will return the encoded 40-bit value as part of a `u64`.
    fn encode_quintuple(&self, decoded_value: &[u8]) -> u64 {
        let mut acc: u64 = 0;

        for i in 0..4 {
            let shift_amount_high = 35 - ((i as u32) * (QUINTUPLE_SIZE * 2) as u32);
            let shift_amount_low = shift_amount_high - QUINTUPLE_SIZE as u32;

            acc |= (self.encode_mappings[(decoded_value[i] >> 4) as usize] as u64)
                << shift_amount_high;
            acc |= (self.encode_mappings[(decoded_value[i] & 0x0F) as usize] as u64)
                << shift_amount_low;
        }

        acc
    }

    /// Encodes the input byte slice (`value`) into a custom encoding format.
    ///
    /// This function processes the input slice in chunks of 4 bytes, encoding each chunk into a new 5-byte segment
    /// by delegating the operation to the `encode_quintuple` method. The resulting encoded chunks are concatenated
    /// into a single vector of bytes.
    ///
    /// # Parameters
    /// - `value`: A slice of bytes (`&[u8]`) representing the data to be encoded.
    ///
    /// # Returns
    /// - `Vec<u8>`: A vector containing the concatenated encoding result of all 4-byte chunks, where each chunk is
    ///   transformed into a 5-byte encoded segment.
    ///
    /// # Details
    /// - The chunking is done using `chunks_exact(4)`, ensuring that only complete chunks of 4 bytes are processed.
    ///   If `value`'s length is not a multiple of 4, the remainder is ignored.
    /// - For each chunk, the `encode_quintuple` method is called to perform the encoding, returning an integer result
    ///   that is then converted into its big-endian byte representation (`to_be_bytes`).
    /// - Only the last 5 bytes of the big-endian representation are used (as the encoded quintuple is presumed to
    ///   require 5 bytes), and these are added to the result vector efficiently using `extend_from_slice`.
    ///
    /// # Example
    /// ```rust
    /// let encoder = Encoder::new();
    /// let input: &[u8] = &[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    /// let output = encoder.encode(input);
    ///
    /// // The output will contain the encoded representation of the first 4 bytes
    /// // and then process additional 4-byte chunks as applicable.
    /// ```
    ///
    /// # Note
    /// - `QUINTUPLE_SIZE` is assumed to be defined elsewhere in the module and represents the fixed size (5 bytes)
    ///   of each encoded segment.
    /// - The `encode_quintuple` method is expected to be implemented for the object type of `self` and should return
    ///   an integer representing the encoded form of a 4-byte chunk.
    ///
    /// # Performance
    /// - The `Vec::with_capacity` is preallocated based on the number of chunks and quintuple size to improve efficiency.
    /// - This method disregards non-complete chunks (remainder of length % 4).
    pub fn encode(&self, value: &[u8]) -> Vec<u8> {
        let num_chunks = value.len() / 4;
        let mut result = Vec::with_capacity(num_chunks * QUINTUPLE_SIZE);

        for chunk in value.chunks_exact(4) {
            let acc = self.encode_quintuple(chunk);
            // Convert to bytes using to_be_bytes and extend efficiently
            result.extend_from_slice(&acc.to_be_bytes()[3..]); // Take last 5 bytes
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_works() {
        let gcr = GCR::new();
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
