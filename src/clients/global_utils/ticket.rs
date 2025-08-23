use std::env;
use std::io::{Cursor, Write, Seek, SeekFrom};
#[cfg(feature = "blowfish")]
use blowfish::Blowfish;
#[cfg(feature = "blowfish")]
use blowfish::cipher::BlockEncrypt;
#[cfg(feature = "blowfish")]
use blowfish::cipher::generic_array::GenericArray;
#[cfg(feature = "blowfish")]
use blowfish::cipher::KeyInit;
#[cfg(feature = "steamworks")]
use steamworks::{AppId, Client, SIResult, networking_types::{NetworkingIdentity}};
#[cfg(feature = "steam")]
use tracing::trace;
#[cfg(feature = "steamworks")]
use crate::clients::global_utils::CrtRand;
#[cfg(feature = "base64")]
use base64::{engine::general_purpose::URL_SAFE};
use base64::Engine;
use crate::error::{Error, Result};


pub(crate) struct Ticket {
    pub(crate) text: String,
    pub(crate) length: u16,
}

const FUCKED_GARBAGE_ALPHABET:&[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";
const SPLIT_SIZE: usize = 300;
impl Ticket {
    #[cfg(feature = "steam")]
    pub(crate) fn new(app_id: AppId) -> Result<Ticket>{
        unsafe {
            env::set_var("SteamAppId", 1.to_string());
            env::set_var("SteamGameId", 1.to_string());
        }

        let c = Client::init_app(app_id)?;

        let ticket = c.user().authentication_session_ticket(NetworkingIdentity::new());
        let time = c.utils().get_server_real_time();

        Ticket::parse(ticket.1, time)
    }
    #[cfg(feature = "steam")]
    pub fn parse(ticket: Vec<u8>, mut time: u32) -> Result<Ticket> {
        // Adjust time
        time -= 5;
        time -= time % 60; // Round to nearest minute

        // Convert ticket to lowercase hex string, then to ASCII bytes
        let ticket_string = ticket.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let mut raw_ticket_bytes = ticket_string.into_bytes();

        // Add null terminator
        let mut raw_ticket = vec![0u8; raw_ticket_bytes.len() + 1];
        raw_ticket[..raw_ticket_bytes.len()].copy_from_slice(&raw_ticket_bytes);
        raw_ticket[raw_ticket_bytes.len()] = 0;

        // Create blowfish key
        let blowfish_key = format!("{:08x}#un@e=x>", time);

        // Create memory buffer
        let mut memory_stream = Cursor::new(Vec::new());

        /* REGULAR SUM + TICKET */
        let mut ticket_sum: u16 = 0;
        for &b in &raw_ticket {
            ticket_sum = ticket_sum.wrapping_add(b as u16);
        }

        // Write ticket sum and raw ticket
        memory_stream.write_all(&ticket_sum.to_le_bytes())?;
        memory_stream.write_all(&raw_ticket)?;

        /* GARBAGE */
        let cast_ticket_sum = ticket_sum as i16; // Equivalent to unchecked cast
        let seed = time ^ (cast_ticket_sum as u32);
        let mut rand = CrtRand::new(seed);

        let num_random_bytes = ((raw_ticket.len() + 9) & 0xFFFFFFFFFFFFFFF8) - 2 - raw_ticket.len();
        let mut garbage = vec![0u8; num_random_bytes];

        // Get fucked_sum from the first 4 bytes of the stream
        let stream_data = memory_stream.get_ref();
        let mut fucked_sum = u32::from_le_bytes([
            stream_data[0], stream_data[1], stream_data[2], stream_data[3]
        ]);

        // Generate garbage bytes
        for i in 0..num_random_bytes {
            let rand_index = ((fucked_sum.wrapping_add(rand.next())) & 0x3F) as usize;
            let rand_char = FUCKED_GARBAGE_ALPHABET[rand_index];
            garbage[i] = rand_char;
            fucked_sum = fucked_sum.wrapping_add(rand_char as u32);
        }

        // Write garbage
        memory_stream.write_all(&garbage)?;

        // Write fucked_sum back to the beginning
        memory_stream.seek(SeekFrom::Start(0))?;
        memory_stream.write_all(&fucked_sum.to_le_bytes())?;

        // Log equivalent (you might want to use your logging crate)
        trace!(
            "[STEAM] time: {}, bfKey: {}, rawTicket.Length: {}, ticketSum: {}, fuckedSum: {}, seed: {}, numRandomBytes: {}",
            time, blowfish_key, raw_ticket.len(), ticket_sum, fucked_sum, seed, num_random_bytes
        );

        /* ENC + SPLIT */
        let mut final_bytes = memory_stream.into_inner();

        // Swap first two bytes
        if final_bytes.len() >= 2 {
            final_bytes.swap(0, 1);
        }

        // Blowfish encryption
        let key_bytes = blowfish_key.into_bytes();
        let blowfish = Blowfish::new_from_slice(&key_bytes)?;

        let mut enc_bytes = vec![0u8; final_bytes.len()];
        debug_assert_eq!(enc_bytes.len() % 8, 0);

        encrypt(blowfish, &final_bytes, &mut enc_bytes);
        let enc_string = URL_SAFE.encode(&enc_bytes).replace("=", "*").to_string();

        // Split into chunks
        let parts: Vec<&str> = chunks_upto(&enc_string, SPLIT_SIZE).collect();
        let final_string = parts.join(",");

        Ok(Ticket {
            text: final_string.clone(),
            length: (final_string.len() - (parts.len() - 1)) as u16,
        })
    }
}

#[cfg(feature = "steam")]
fn encrypt(blowfish: Blowfish, input: &[u8], output: &mut [u8]) {
    for s in (0..input.len()).step_by(8) {
        let input = GenericArray::from_slice(&input[s..s + 8]);

        let out = GenericArray::from_mut_slice(&mut output[s..s + 8]);
        blowfish.encrypt_block_b2b(input, out);
    }
}


// Helper function to split string into chunks
fn chunks_upto(s: &str, chunk_size: usize) -> impl Iterator<Item = &str> {
    (0..s.len())
        .step_by(chunk_size)
        .map(move |i| {
            let end = std::cmp::min(i + chunk_size, s.len());
            &s[i..end]
        })
}