use std::collections::VecDeque;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const MAX_BODY_BYTES: usize = 256 * 1024;
const IO_BUDGET_BYTES: usize = 64 * 1024;
const MAX_IO_ATTEMPTS: usize = 64;
const MAX_OUTPUT_FRAMES: usize = 2;
const OUTPUT_DEADLINE: Duration = Duration::from_secs(1);

struct OutputFrame {
    header: [u8; 4],
    body: Vec<u8>,
    sent: usize,
}

pub struct FramedTcp {
    stream: TcpStream,
    header: [u8; 4],
    header_read: usize,
    body: Vec<u8>,
    body_read: usize,
    output: VecDeque<OutputFrame>,
    last_write_progress: Option<Instant>,
    failure: Option<(ErrorKind, String)>,
}

impl FramedTcp {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            header: [0; 4],
            header_read: 0,
            body: Vec::new(),
            body_read: 0,
            output: VecDeque::with_capacity(MAX_OUTPUT_FRAMES),
            last_write_progress: None,
            failure: None,
        })
    }

    pub fn poll_read(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.check_open()?;
        let mut budget = IO_BUDGET_BYTES;
        for _ in 0..MAX_IO_ATTEMPTS {
            if budget == 0 {
                break;
            }
            if self.header_read < self.header.len() {
                let end = self.header.len().min(self.header_read + budget);
                match self.stream.read(&mut self.header[self.header_read..end]) {
                    Ok(0) => return self.read_eof(),
                    Ok(count) => {
                        self.header_read += count;
                        budget -= count;
                        if self.header_read == self.header.len() {
                            let length = u32::from_le_bytes(self.header) as usize;
                            if length == 0 || length > MAX_BODY_BYTES {
                                return self.fail(io::Error::new(
                                    ErrorKind::InvalidData,
                                    "Invalid TCP frame body length",
                                ));
                            }
                            self.body = vec![0; length];
                        }
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
                    Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                    Err(error) => return self.fail(error),
                }
            } else {
                let end = self.body.len().min(self.body_read + budget);
                match self.stream.read(&mut self.body[self.body_read..end]) {
                    Ok(0) => return self.read_eof(),
                    Ok(count) => {
                        self.body_read += count;
                        budget -= count;
                        if self.body_read == self.body.len() {
                            self.header_read = 0;
                            self.body_read = 0;
                            return Ok(Some(std::mem::take(&mut self.body)));
                        }
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
                    Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                    Err(error) => return self.fail(error),
                }
            }
        }
        Ok(None)
    }

    pub fn queue(&mut self, body: Vec<u8>) -> io::Result<()> {
        self.check_open()?;
        if body.is_empty() || body.len() > MAX_BODY_BYTES {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "TCP frame body must contain between 1 and 262144 bytes",
            ));
        }
        if self.output.len() >= MAX_OUTPUT_FRAMES {
            return Err(io::Error::new(
                ErrorKind::WouldBlock,
                "TCP output queue is full",
            ));
        }
        if self.output.is_empty() {
            self.last_write_progress = Some(Instant::now());
        }
        self.output.push_back(OutputFrame {
            header: (body.len() as u32).to_le_bytes(),
            body,
            sent: 0,
        });
        Ok(())
    }

    pub fn poll_write(&mut self) -> io::Result<()> {
        self.check_open()?;
        let mut budget = IO_BUDGET_BYTES;
        for _ in 0..MAX_IO_ATTEMPTS {
            if budget == 0 || self.output.is_empty() {
                break;
            }
            if self
                .last_write_progress
                .is_some_and(|progress| progress.elapsed() >= OUTPUT_DEADLINE)
            {
                return self.fail(io::Error::new(
                    ErrorKind::TimedOut,
                    "TCP output made no progress for one second",
                ));
            }
            let written = {
                let frame = self.output.front().expect("output is not empty");
                let pending = if frame.sent < frame.header.len() {
                    &frame.header[frame.sent..]
                } else {
                    &frame.body[frame.sent - frame.header.len()..]
                };
                self.stream.write(&pending[..pending.len().min(budget)])
            };
            match written {
                Ok(0) => {
                    return self.fail(io::Error::new(
                        ErrorKind::WriteZero,
                        "TCP write made no progress",
                    ));
                }
                Ok(count) => {
                    budget -= count;
                    self.last_write_progress = Some(Instant::now());
                    let frame = self.output.front_mut().expect("output is not empty");
                    frame.sent += count;
                    if frame.sent == frame.header.len() + frame.body.len() {
                        self.output.pop_front();
                        if self.output.is_empty() {
                            self.last_write_progress = None;
                        }
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return self.fail(error),
            }
        }
        Ok(())
    }

    pub fn is_output_empty(&self) -> bool {
        self.output.is_empty()
    }

    fn check_open(&self) -> io::Result<()> {
        self.failure.as_ref().map_or(Ok(()), |(kind, message)| {
            Err(io::Error::new(*kind, message.clone()))
        })
    }

    fn read_eof(&mut self) -> io::Result<Option<Vec<u8>>> {
        let message = if self.header_read == 0 {
            "TCP peer closed the connection"
        } else {
            "TCP peer closed during a frame"
        };
        self.fail(io::Error::new(ErrorKind::UnexpectedEof, message))
    }

    fn fail<T>(&mut self, error: io::Error) -> io::Result<T> {
        self.failure = Some((error.kind(), error.to_string()));
        Err(error)
    }
}
