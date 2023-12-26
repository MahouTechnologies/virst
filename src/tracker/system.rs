use std::{
    io::ErrorKind,
    net::{ToSocketAddrs, UdpSocket},
    panic::{catch_unwind, UnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
    time::Duration,
};

use thiserror::Error;
use mahou_vmc::VmcData;

pub struct TrackerSystem {
    join_handle: Option<JoinHandle<()>>,
    abort: Arc<AtomicBool>,
    data: Arc<Mutex<VmcData>>,
}

#[derive(Error, Debug)]
#[error("tracker subsystem is already connected")]
pub struct AlreadyConnectedError;

impl TrackerSystem {
    pub fn new() -> Self {
        TrackerSystem {
            join_handle: None,
            abort: Arc::new(AtomicBool::new(false)),
            data: Arc::new(Mutex::new(VmcData::default())),
        }
    }

    pub fn data(&self) -> &Mutex<VmcData> {
        &self.data
    }

    pub fn reset(&self) {
        let mut data = self.data.lock().unwrap();
        *data = VmcData::default();
    }

    pub fn disconnect(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            self.abort.store(true, Ordering::Relaxed);
            handle.join().expect("joining network thread");
        }
    }

    pub fn active(&self) -> bool {
        self.join_handle
            .as_ref()
            .map(|x| !x.is_finished())
            .unwrap_or(false)
    }

    pub fn connect<A: ToSocketAddrs + Send + UnwindSafe + 'static>(
        &mut self,
        addr: A,
    ) -> Result<(), AlreadyConnectedError> {
        if self.join_handle.is_some() {
            return Err(AlreadyConnectedError);
        }
        self.abort.store(false, Ordering::Relaxed);

        let data = Arc::clone(&self.data);
        let abort = Arc::clone(&self.abort);
        let handle = spawn(move || {
            let e = catch_unwind(|| {
                let sock = UdpSocket::bind(addr).unwrap();
                sock.set_read_timeout(Some(Duration::from_millis(500)))
                    .expect("Setting timeout won't fail");

                loop {
                    if abort.load(Ordering::Relaxed) {
                        break;
                    }

                    let mut buf = [0u8; 65536];
                    match sock.recv_from(&mut buf) {
                        Ok((size, _)) => {
                            let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                            data.lock().unwrap().update_from_packet(packet);
                        }
                        Err(e) => {
                            if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock
                            {
                                continue;
                            }
                            println!("Error receiving from socket: {}", e);
                        }
                    }
                }
            });
            if let Err(e) = e {
                eprintln!("{:?}", e);
            }
        });

        self.join_handle = Some(handle);
        Ok(())
    }
}
