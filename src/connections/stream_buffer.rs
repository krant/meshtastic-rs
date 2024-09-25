use crate::protobufs;
use prost::Message;
use tokio::sync::mpsc::UnboundedSender;

use super::wrappers::encoded_data::IncomingStreamData;

/// A struct that represents a buffer of bytes received from a radio stream.
/// This struct is used to store bytes received from a radio stream, and is
/// used to incrementally decode bytes from the received stream into valid
/// FromRadio packets.
#[derive(Clone, Debug)]
pub struct StreamBuffer {
    buffer: Vec<u8>,
    decoded_packet_tx: UnboundedSender<protobufs::FromRadio>,
}


impl StreamBuffer {
    /// Creates a new StreamBuffer instance that will send decoded FromRadio packets
    /// to the given broadcast channel.
    pub fn new(decoded_packet_tx: UnboundedSender<protobufs::FromRadio>) -> Self {
        StreamBuffer {
            buffer: vec![],
            decoded_packet_tx,
        }
    }

    /// Takes in a portion of a stream message, stores it in a buffer,
    /// and attempts to decode the buffer into valid FromRadio packets.
    ///
    /// # Arguments
    ///
    /// * `message` - A vector of bytes received from a radio stream
    ///
    /// # Example
    ///
    /// ```
    /// let (rx, mut tx) = broadcast::channel::<protobufs::FromRadio>(32);
    /// let buffer = StreamBuffer::new(tx);
    ///
    /// while let Some(message) = stream.try_next().await? {
    ///    buffer.process_incoming_bytes(message);
    /// }
    /// ```
    pub fn process_incoming_bytes(&mut self, message: IncomingStreamData) {
        let buf = &mut self.buffer;
        for b in  message.data() {
            let ptr = buf.len();
            buf.push(*b);
            if ptr == 0 {
                if *b != 0x94 {
                    //print!("{}", *b as char);
                    buf.clear();
                }
            } else if ptr == 1 {
                if *b != 0xc3 {
                    buf.clear();
                }
            } else if ptr >= 3 {
                let plen = u16::from_be_bytes([buf[2], buf[3]]) as usize;
                if ptr == 3 && plen > 512 {
                    buf.clear();
                }
                if buf.len() != 0 && ptr + 1 >= plen + 4 {
                    if let Ok(packet) = protobufs::FromRadio::decode(&buf[4..]) {
                        let _ = self.decoded_packet_tx.send(packet);
                    }
                    buf.clear();
                }
            }
        }
    }
}
