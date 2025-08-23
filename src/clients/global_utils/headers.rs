use std::sync::LazyLock;
use reqwest::{header, RequestBuilder};
use sha1::{Digest, Sha1};

static USERAGENT: LazyLock<String> = LazyLock::new(|| {
    format!("SQEXAuthor/2.0.0(Windows 6.2; ja-jp; {})", make_computer_id())
});


pub(crate) trait DefaultHeaders {
    fn default_ffxiv_headers(self) -> Self;
}

impl DefaultHeaders for RequestBuilder {
    fn default_ffxiv_headers(self) -> Self {
        self.header(header::ACCEPT, "image/gif, image/jpeg, image/pjpeg, application/x-ms-application, application/xaml+xml, application/x-ms-xbap, */*")
            .header(header::ACCEPT_ENCODING, "gzip, deflate")
            .header(header::ACCEPT_LANGUAGE, "en-US")
            .header(header::USER_AGENT, &*USERAGENT)
            .header(header::CONNECTION, "Keep-Alive")
            .header(header::UPGRADE_INSECURE_REQUESTS, "true")
    }
}


fn make_computer_id() -> String {
    // Get system information (you'll need appropriate crates for cross-platform support)
    let machine_name = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let user_name = whoami::username();

    let os_version = format!("{} {}",
                             std::env::consts::OS,
                             std::env::consts::ARCH);

    let processor_count = num_cpus::get().to_string();

    // Create hash string by concatenating system info
    let hash_string = format!("{}{}{}{}",
                              machine_name, user_name.unwrap_or("username".to_string()), os_version, processor_count);

    // Convert to UTF-16 bytes (equivalent to C#'s Encoding.Unicode)
    let utf16_bytes: Vec<u8> = hash_string
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    // Compute SHA1 hash
    let mut hasher = Sha1::new();
    hasher.update(&utf16_bytes);
    let hash_result = hasher.finalize();

    // Create 5-byte array
    let mut bytes = [0u8; 5];

    // Copy first 4 bytes of hash to positions 1-4
    bytes[1..5].copy_from_slice(&hash_result[0..4]);

    // Calculate checksum (negative sum of bytes 1-4) with wrapping arithmetic
    let sum = bytes[1]
        .wrapping_add(bytes[2])
        .wrapping_add(bytes[3])
        .wrapping_add(bytes[4]);
    bytes[0] = sum.wrapping_neg();

    // Convert to lowercase hex string
    hex::encode(bytes)
}