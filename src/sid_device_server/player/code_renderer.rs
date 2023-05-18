// Copyright (C) 2023 Brian Holdsworth
// Licensed under the GNU GPL v3 license. See the LICENSE file for the terms and conditions.

use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{thread, time::{Duration, Instant}};

use atomicring::AtomicRingBuffer;
use crossbeam_channel::{Sender, Receiver, bounded};
use typed_builder::TypedBuilder;

use thread_priority::{set_current_thread_priority, ThreadPriority};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crate::sid_device_server::player::{PlayerCommand, SidWrite};

use std::os::unix::net::UnixDatagram;
pub const NMIPORT: &str = "/tmp/idunmm-nmi";

pub static CODER_ERROR: AtomicBool = AtomicBool::new(false);

const PAL_CLOCK: u32 = 985_248;
const NTSC_CLOCK: u32 = 1_022_727;
const PAUSE_AUDIO_IDLE_TIME_IN_SEC: u64 = 2;
const CYCLES_IN_BUFFER_THRESHOLD: u32 = 10_000;
const CODE_BLOCK_SIZE: usize = 512;
const CODE_BLOCKS_MAX_LIMIT: usize = 5_000;
const STOP_PAUSE_LATENCY_IN_MILLIS: u64 = 10;

struct DeviceState {
    should_stop: Arc<AtomicBool>,
    should_pause: Arc<AtomicBool>,
    queue_started: Arc<AtomicBool>,
    aborted: Arc<AtomicBool>,
    cycles_in_buffer: Arc<AtomicU32>
}

#[derive(TypedBuilder)]
pub struct Config {
    pub clock: u32,
    pub position_left: Vec<i32>,
    pub position_right: Vec<i32>,

    #[builder(default=false)]
    pub config_changed: bool
}

pub struct CodeRenderer {
    in_cmd_sender: Sender<(PlayerCommand, Option<i32>)>,
    in_cmd_receiver: Receiver<(PlayerCommand, Option<i32>)>,
    out_sid_read_sender: Sender<u8>,
    out_sid_read_receiver: Receiver<u8>,
    queue: Arc<AtomicRingBuffer<SidWrite>>,
    queue_started: Arc<AtomicBool>,
    aborted: Arc<AtomicBool>,
    cycles_in_buffer: Arc<AtomicU32>,
    should_stop_producer: Arc<AtomicBool>,
    should_stop_generator: Arc<AtomicBool>,
    should_pause: Arc<AtomicBool>,
    coder_thread: Option<thread::JoinHandle<()>>,
    idunmm_thread: Option<thread::JoinHandle<()>>,
    config: Arc<Mutex<Config>>,
    code_buffer: Arc<AtomicRingBuffer<Vec<u8>>>
}

impl Drop for CodeRenderer {
    fn drop(&mut self) {
        self.stop_threads();
    }
}

impl CodeRenderer {
    pub fn new(
        queue: Arc<AtomicRingBuffer<SidWrite>>,
        queue_started: Arc<AtomicBool>,
        aborted: Arc<AtomicBool>,
        cycles_in_buffer: Arc<AtomicU32>
    ) -> CodeRenderer {
        let (in_cmd_sender, in_cmd_receiver) = bounded(0);
        let (out_sid_read_sender, out_sid_read_receiver) = bounded(0);
        let should_stop_producer = Arc::new(AtomicBool::new(false));
        let should_stop_generator = Arc::new(AtomicBool::new(false));
        let should_pause = Arc::new(AtomicBool::new(false));
        let config = Self::create_default_config();
        let code_buffer = Arc::new(AtomicRingBuffer::<Vec<u8>>::with_capacity(CODE_BLOCKS_MAX_LIMIT));

        CodeRenderer {
            in_cmd_sender,
            in_cmd_receiver,
            out_sid_read_sender,
            out_sid_read_receiver,
            queue,
            queue_started,
            aborted,
            cycles_in_buffer,
            should_stop_producer,
            should_stop_generator,
            should_pause,
            coder_thread: None,
            idunmm_thread: None,
            config: Arc::new(Mutex::new(config)),
            code_buffer
        }
    }

    pub fn stop_threads(&mut self) {
        self.stop_generator_thread();
        self.stop_producer_thread();
    }

    fn stop_generator_thread(&mut self) {
        self.should_stop_generator.store(true, Ordering::SeqCst);

        if self.coder_thread.is_some() {
            let _ = self.coder_thread.take().unwrap().join().ok();
        }

        self.should_stop_generator.store(false, Ordering::SeqCst);
    }


    fn stop_producer_thread(&mut self) {
        self.should_stop_producer.store(true, Ordering::SeqCst);

        if self.idunmm_thread.is_some() {
            let _ = self.idunmm_thread.take().unwrap().join().ok();
        }

        self.should_stop_producer.store(false, Ordering::SeqCst);
    }

    pub fn start(&mut self) {
        let mut restart = self.idunmm_thread.is_some() || self.coder_thread.is_some();
        self.stop_threads();

        if CODER_ERROR.load(Ordering::SeqCst) {
            CODER_ERROR.store(false, Ordering::SeqCst);
            restart = false;
        }

        self.start_idunmm_thread(!restart);

        let mut config = self.config.clone();

        let mut code_buffer_clone = self.code_buffer.clone();
        let should_stop_generator_clone = self.should_stop_generator.clone();
        let should_pause_clone = self.should_pause.clone();
        let aborted = self.aborted.clone();
        let mut queue = self.queue.clone();
        let cycles_in_buffer = self.cycles_in_buffer.clone();

        let in_cmd_receiver = self.in_cmd_receiver.clone();
        let out_sid_read_sender = self.out_sid_read_sender.clone();

        let queue_started = self.queue_started.clone();

        let device_state = DeviceState {
            should_stop: should_stop_generator_clone,
            should_pause: should_pause_clone,
            queue_started,
            aborted,
            cycles_in_buffer
        };

        self.coder_thread = Some(thread::spawn(move || {
            Self::sid_coder_thread(
                &mut queue,
                &in_cmd_receiver,
                &out_sid_read_sender,
                &mut config,
                &mut code_buffer_clone,
                device_state
            )
        }));
    }

    fn start_idunmm_thread(&mut self, _restart: bool) {
        let should_stop_producer_clone = self.should_stop_producer.clone();
        let should_pause = self.should_pause.clone();
        let code_buffer_clone = self.code_buffer.clone();

        self.idunmm_thread = Some(thread::spawn(move || {
            run(code_buffer_clone, should_stop_producer_clone, should_pause);
        }));
    }

    fn sid_coder_thread(
        queue: &mut Arc<AtomicRingBuffer<SidWrite>>,
        in_cmd_receiver_clone: &Receiver<(PlayerCommand, Option<i32>)>,
        _out_sid_read_sender: &Sender<u8>,
        config: &mut Arc<Mutex<Config>>,
        code_buffer: &mut Arc<AtomicRingBuffer<Vec<u8>>>,
        device_state: DeviceState
    ) {
        let _ = set_current_thread_priority(ThreadPriority::Max);

        let mut config = config.lock();

        let mut last_activity = Instant::now();
        loop {
            if device_state.should_stop.load(Ordering::SeqCst) {
                break;
            }
            if device_state.aborted.load(Ordering::SeqCst) {
                code_buffer.clear();
                device_state.aborted.store(false, Ordering::SeqCst);
            }

            if !queue.is_empty() && device_state.queue_started.load(Ordering::SeqCst) {
                last_activity = Instant::now();
                device_state.should_pause.store(false, Ordering::SeqCst);
            } else if !device_state.should_pause.load(Ordering::SeqCst) && last_activity.elapsed().as_secs() > PAUSE_AUDIO_IDLE_TIME_IN_SEC {
                device_state.should_pause.store(true, Ordering::SeqCst);
            }

            let cmd = process_player_command(in_cmd_receiver_clone, &mut config);

            if let Some((command, _param1)) = cmd {
                if command == PlayerCommand::Read {
                    while !queue.is_empty() {
                        generate_code(code_buffer, queue, &device_state.cycles_in_buffer, &mut config);
                    }

                    // let reg = param1.unwrap_or(0);
                    // let sid_num = min(reg >> 5, config.sid_count - 1) as usize;

                    // let sid_env_out = sids[sid_num].read(reg as u32 & 0x1f) as u8;
                    // let _ = out_sid_read_sender.send(sid_env_out);
                }
            } else {
                if !device_state.queue_started.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }

                try_generate_code(code_buffer, queue, &device_state.cycles_in_buffer, &mut config);
                if Self::has_enough_data(&device_state) {
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }

    #[inline]
    fn has_enough_data(device_state: &DeviceState) -> bool {
        device_state.cycles_in_buffer.load(Ordering::SeqCst) > CYCLES_IN_BUFFER_THRESHOLD
    }

    fn create_default_config() -> Config {
        Config::builder()
            .clock(PAL_CLOCK)
            .position_left(vec![0])
            .position_right(vec![0])
            .build()
    }

    pub fn get_channel_sender(&self) -> Sender<(PlayerCommand, Option<i32>)> {
        self.in_cmd_sender.clone()
    }

    pub fn get_sid_read_receiver(&self) -> Receiver<u8> {
        self.out_sid_read_receiver.clone()
    }
}

#[inline]
fn process_player_command(in_cmd_receiver: &Receiver<(PlayerCommand, Option<i32>)>, config: &mut Config) -> Option<(PlayerCommand, Option<i32>)> {
    let recv_result = in_cmd_receiver.try_recv();

    if let Ok((command, param1)) = recv_result {
        match command {
            PlayerCommand::SetModel => {
                    config.config_changed = true;
            }
            PlayerCommand::SetClock => {
                let clock = param1.unwrap();
                config.clock = if clock == 0 {
                    PAL_CLOCK
                } else {
                    NTSC_CLOCK
                };

                config.config_changed = true;
            }
            PlayerCommand::SetSidCount => {
                config.config_changed = true;
            }
            PlayerCommand::SetPosition => {
                if let Some(param1) = param1 {
                    let position = ((param1 & 0xff) as i8) as i32;
                    let sid_number = param1 >> 8;
                    if sid_number == 1 {
                        config.position_left[sid_number as usize] = if position <= 0 { 100 } else { 100 - position };
                        config.position_right[sid_number as usize] = if position >= 0 { 100 } else { 100 + position };
                    }
                }
            }
            PlayerCommand::SetSamplingMethod => {
                config.config_changed = true;
            }
            PlayerCommand::SetMute => {
                // Turn off all SID channels
                let sock = UnixDatagram::unbound().unwrap();
                let code = vec![0x09u8, 0x80, 0x09, 0x80, 0xc3, 0xc2, 0xcd, 0x31, 0x30,
                    0xa9, 0x00, 0x8d, 0x00, 0xd4, 0x8d, 0x01, 0xd4,
                                0x8d, 0x07, 0xd4, 0x8d, 0x08, 0xd4,
                                0x8d, 0x0e, 0xd4, 0x8d, 0x0f, 0xd4, 0x60];
                send_code_block(&sock, code);
            }
            PlayerCommand::Reset => {
                config.config_changed = true;
            }
            _ => {}
        }
        return Some((command, param1));
    }
    None
}

fn configure_sid(config: &mut Config) {
    config.config_changed = false;
}

fn try_generate_code(output_stream: &mut Arc<AtomicRingBuffer<Vec<u8>>>, sid_write_queue: &mut Arc<AtomicRingBuffer<SidWrite>>, cycles_in_buffer: &Arc<AtomicU32>, config: &mut Config) {
    if sid_write_queue.len() > 67 && output_stream.len() < CODE_BLOCKS_MAX_LIMIT {
        generate_code(output_stream, sid_write_queue, cycles_in_buffer, config);
    }
}

fn generate_delay(cycles: u32, clock: u32, stream: &mut Arc<AtomicRingBuffer<Vec<u8>>>) {
    let mut delay = vec![];
    let c: u64 = (cycles as u64) * (clock as u64) / 1_000_000;
    delay.write_u32::<BigEndian>(c as u32).unwrap();
    stream.try_push(delay).ok();
}

fn generate_write_reg(code: &mut Vec<u8>, write: SidWrite) {
    code.push(0xa9);         //LDA imm
    code.push(write.data);
    code.push(0x8d);         //STA abs
    code.push(write.reg & 0x1f);
    code.push(0xd4);
}

fn generate_send_code(code: &mut Vec<u8>, stream: &mut Arc<AtomicRingBuffer<Vec<u8>>>)
{
    let clen = code.len();
    // Push code buffer to output stream
    if clen > 0 {
        if clen > (CODE_BLOCK_SIZE-1) {
            eprintln!("Oversize code block {} bytes. Truncating.", code.len());
            code.truncate(CODE_BLOCK_SIZE-1);
        }
        code.push(0x60);     //RTS
        stream.try_push(code.to_vec()).ok();
        code.clear();
        code.extend(&[0x09, 0x80, 0x09, 0x80, 0xc3, 0xc2, 0xcd, 0x31, 0x30]);
    }
}

fn generate_code(output_stream: &mut Arc<AtomicRingBuffer<Vec<u8>>>, sid_write_queue: &mut Arc<AtomicRingBuffer<SidWrite>>, cycles_in_buffer: &Arc<AtomicU32>, config: &mut Config) {
    if output_stream.len() > CODE_BLOCKS_MAX_LIMIT {
        return;
    }

    if config.config_changed {
        configure_sid(config);
    }

    let mut total_cycles: u32 = 0;
    let mut code_buffer = Vec::<u8>::with_capacity(CODE_BLOCK_SIZE);
	code_buffer.extend(&[0x09, 0x80, 0x09, 0x80, 0xc3, 0xc2, 0xcd, 0x31, 0x30]);

    loop {
        let mut sid_write = sid_write_queue.try_pop();
        let mut cycles: u32 = 0;
        let mut total_bytes = code_buffer.len() as u32;
        loop {
            if let Some(sidwr) = sid_write {

                cycles = sidwr.cycles as u32;
                total_cycles = total_cycles + cycles;

                if cycles <= 20 {
                    // insert NOPs to burn excess cycles
                    let mut nop = 0;
                    if cycles > 6 { nop = (cycles-6)/2 }

                    let needed_bytes = nop + 5;
                    if (total_bytes+needed_bytes) < CODE_BLOCK_SIZE as u32 {
                        code_buffer.extend(vec![0xea; nop as usize].as_slice());
                        generate_write_reg(&mut code_buffer, sidwr);
                        total_bytes = total_bytes + needed_bytes;
                        sid_write = sid_write_queue.try_pop();
                        continue;
                    } else {
                        total_cycles = total_cycles - cycles;
                        sid_write_queue.try_push(sidwr).ok();
                    }
                } else if cycles <= 1026 {
                    // insert loop delay to burn excess cycles
                    if (total_bytes+10) < CODE_BLOCK_SIZE as u32 {
                        let mut looper = [0xa2u8,0x01,0xca,0xd0,0xfd];
                        looper[1] = ((cycles-6) / 4) as u8;
                        code_buffer.extend(&looper);
                        generate_write_reg(&mut code_buffer, sidwr);
                        total_bytes = total_bytes + 10;
                        sid_write = sid_write_queue.try_pop();
                        continue;
                    } else {
                        total_cycles = total_cycles - cycles;
                        sid_write_queue.try_push(sidwr).ok();
                    }
                }
                break;
            } else {
                break;
            }
        }

        // Push current code buffer
        generate_send_code(&mut code_buffer, output_stream);
        if sid_write.is_some() && sid_write.unwrap().cycles > 1026 {
            generate_delay(cycles-6, config.clock, output_stream);
            generate_write_reg(&mut code_buffer, sid_write.unwrap());
            continue;
        }

        // Exit because no more writes in queue
        break;
    }            

    if total_cycles > 0 {
        let cycles = cycles_in_buffer.load(Ordering::SeqCst);
        if cycles > total_cycles {
            cycles_in_buffer.fetch_sub(total_cycles, Ordering::SeqCst);
        } else {
            cycles_in_buffer.store(0, Ordering::SeqCst);
        }
    }
}

fn send_code_block(socket: &UnixDatagram, block: Vec<u8>) {
    let mut hdr = Vec::<u8>::new();
    hdr.write_u16::<BigEndian>(block.len() as u16).unwrap();
    // Send the two-byte size hdr
    let path = Path::new(NMIPORT);
    match socket.send_to(&hdr, path) {
        // Send the code block, or panic for any failure...
        Ok(2) => {
            //eprintln!("nmi msg: {:x?}", code);
            let code_len = socket.send_to(&block, path).expect("Failed nmi command");
            assert!(code_len == block.len());
        },
        Ok(_) => panic!("Failed send NMI command"),
        Err(e) => panic!("{}", e),
    }
}

fn run(code_blocks: Arc<AtomicRingBuffer<Vec<u8>>>, should_stop: Arc<AtomicBool>, should_pause: Arc<AtomicBool>)
{
    let sock = UnixDatagram::unbound().unwrap();

    while !should_stop.load(Ordering::SeqCst) {
        loop {
            let blk = code_blocks.try_pop();
            if let Some(code) = blk {
                // Check if it's a delay?
                if code[0] == 0 {
                    let delay = code.as_slice().read_u32::<BigEndian>().unwrap();
                    //eprintln!("Delay {} micros", delay);
                    thread::sleep(Duration::from_micros(delay as u64));
                } else {
                    send_code_block(&sock, code);
                }
            } else {
                break;
            }
        }

        while should_pause.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(STOP_PAUSE_LATENCY_IN_MILLIS));
            if should_stop.load(Ordering::SeqCst) {
                break;
            }
        }

        thread::sleep(Duration::from_millis(STOP_PAUSE_LATENCY_IN_MILLIS));
    }

}
