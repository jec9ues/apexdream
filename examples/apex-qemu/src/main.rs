use std::{str, thread};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, TryRecvError};
use dataview::Pod;
use egui_backend::egui::{
    Align2, Color32, FontId, Galley, LayerId, Painter, Pos2, Rect, Rounding, Shape, Stroke, TextureId,
};
use egui_backend::egui::epaint::{CircleShape, RectShape, TextShape};
use egui_backend::egui::Shape::Text;
use egui_backend::egui::text::Fonts;
use fmtools::fmt;
use intptr::IntPtr as Ptr;
use memprocfs;
use memprocfs::{
    CONFIG_OPT_REFRESH_ALL, FLAG_NOCACHE, FLAG_NOPAGING, FLAG_ZEROPAD_ON_FAIL, ResultEx, Vmm,
    VmmProcess,
};
use mouse_rs::Mouse;
use obfstr::obfstr as s;

use apexdream::*;
use apexdream::Instance;
use apexdream::Interface;
use kmbox_net::KmboxNet;
use overlay::get_context;

mod overlay;

fn apex_legends(rt: &mut Runtime) -> bool {
    rt.log(fmt!("Apex Legends"));
    let Ok(gd) = std::fs::read_to_string(s!("gamedata_v3.0.62.30.json")) else {
        rt.log(fmt!("Error reading gamedata file"));
        return false;
    };

    let mut inst = Instance::default();
    if inst.attach(rt, &gd) {
        while rt.heartbeat() {
            inst.tick(rt);
            rt.tick();
        }
        let signal = rt.signal;
        rt.log(fmt!("SignalExit("{signal}")"));
    } else {
        rt.log(fmt!("Error Instance::new"));
    }
    rt.signal == 2
}

fn main() {
    let vmm_args = ["-waitinitialize", "-device", "fpga", "-memmap", "auto"].to_vec();

    let vmm = Vmm::new(r"D:\Driver\memprocfs\vmm.dll", &vmm_args).unwrap();
    let vp = vmm.process_from_name("r5apex.exe");
    let _r = vmm.set_config(CONFIG_OPT_REFRESH_ALL, 1);

    let (sx, rx) = bounded::<egui_backend::egui::Context>(1);
    thread::spawn(move || get_context(sx.clone()));
    let mut context: Option<egui_backend::egui::Context>;
    loop {
        match rx.try_recv() {
            Ok(con) => {
                context = Some(con);
                break;
            }
            Err(_) => {}
        }
    }

    let context = context.unwrap();
    let ptr = Painter::new(context.clone(), LayerId::debug(), Rect::EVERYTHING);


    let runtime = tokio::runtime::Runtime::new().unwrap();
    let client = runtime.block_on(KmboxNet::get_kmbox_client());

    let mut rt = Runtime {
        vmm: &vmm,
        vp,
        signal: 0,
        context: context,
        draw_ptr: ptr,
        kmbox_client: client,
        tokio_runtime: runtime,
    };

    apex_legends(&mut rt);

    sleep(Duration::from_secs(1))
}

struct Runtime<'a> {
    vmm: &'a Vmm<'a>,
    vp: ResultEx<VmmProcess<'a>>,
    signal: u32,
    context: egui_backend::egui::Context,
    draw_ptr: Painter,
    kmbox_client: KmboxNet,
    tokio_runtime: tokio::runtime::Runtime,
}

impl Runtime<'_> {
    fn heartbeat(&self) -> bool {
        if self.signal != 0 {
            return false;
        }

        let Ok(process) = &self.vp else {
            return false;
        };

        match process.info() {
            Ok(i) => {
                if i.state == 0 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn tick(&mut self) {
        let mut signal = match &self.vp {
            Ok(i) => i.info().unwrap().state,
            _ => 2,
        };

        if signal > 0 {
            match self.vmm.process_from_name("r5apex.exe") {
                Ok(vp) => self.vp = Ok(vp),
                Err(_) => {
                    self.signal = signal;
                    return;
                }
            };
        };
    }

    fn log(&mut self, args: impl std::fmt::Display) {
        Interface::log(self, format_args!("{}", args));
    }
}

#[allow(unused_variables)]
impl Interface for Runtime<'_> {
    fn get_time(&mut self) -> f64 {
        use winapi::*;
        use winapi::um::profileapi::QueryPerformanceCounter;
        use winapi::um::profileapi::QueryPerformanceFrequency;
        static mut TIME_BASE: u64 = 0;

        /// Returns the time in seconds since the first time this function was called.
        pub fn time_s() -> f64 {
            unsafe {
                let mut counter = 0u64;
                let mut frequency = 0u64;
                QueryPerformanceCounter(&mut counter as *mut _ as *mut _);
                QueryPerformanceFrequency(&mut frequency as *mut _ as *mut _);
                if TIME_BASE == 0 {
                    TIME_BASE = counter;
                    0.0
                } else {
                    (counter - TIME_BASE) as f64 / frequency as f64
                }
            }
        }

        time_s()
    }

    fn sleep(&mut self, ms: u32) {
        sleep(Duration::from_millis(ms as u64));
    }

    fn log(&mut self, args: std::fmt::Arguments) {
        println!("{}", args);
    }

    fn visualize(&mut self, scope: &str, args: std::fmt::Arguments) {
        // println!("{}", args);
    }

    fn dump_bin(&mut self, path: &str, data: &[u8]) {
        match std::fs::write(path, data) {
            Ok(_) => (),
            Err(err) => self.log(fmt!("Failed to write "{path}": "{err})),
        }
    }


    fn mouse_move(&mut self, dx: i32, dy: i32) {
        self.tokio_runtime.block_on(self.kmbox_client.mouse_move(dx, dy));
    }

    /*    fn mouse_move(&mut self, pitch: f32, yaw: f32, view: u64) {
            // self.write_memory(view, dataview::bytes(&[pitch, yaw]));
            // self.mouse_client.move_to(dx, dy).expect("Unable to move mouse");
        }*/

    fn base_address(&mut self) -> u64 {
        let Ok(vp) = &self.vp else {
            return 0u64;
        };
        let Ok(base) = vp.get_module_base("r5apex.exe") else {
            return 0u64;
        };
        base
    }

    fn read_memory(&mut self, address: u64, dest: &mut [u8]) -> i32 {
        let Ok(vp) = &self.vp else {
            return -1;
        };
        match vp.mem_read_ex(
            address,
            dest.len(),
            FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL | FLAG_NOPAGING,
        ) {
            Ok(data) => {
                dest.copy_from_slice(&data);
                0
            }
            Err(_) => -1,
        }
    }

    // Pasted from some other project, I hope it works :)
    // Can probably be implemented a little smarter on Win32
    fn gather_memory(&mut self, base_address: u64, size: u32, indices: &mut [u32]) -> i32 {
        let mut buf = [0u8; 0x1000];

        // Keep track of indices read within reasonable limit
        if indices.len() >= 128 {
            return -1;
        }
        let mut read_mask = 0u128;

        // For every index
        let mut success = false;
        for i in 0..indices.len() {
            if read_mask & (1u128 << i) == 0 {
                let virtual_address = (base_address + indices[i] as u64) & !0xfff;
                let temp = if self.read_memory(virtual_address, &mut buf) >= 0 {
                    // If a single read was succesful the whole read is successful
                    success = true;
                    Some(&buf)
                } else {
                    None
                };

                // Read all indices in the page
                for j in i..indices.len() {
                    if read_mask & (1u128 << j) == 0 {
                        let index_address = base_address + indices[j] as u64;
                        if index_address >= virtual_address
                            && index_address < virtual_address + 0x1000
                        {
                            // Mark the index as read
                            read_mask |= 1u128 << j;

                            // Try to read the index
                            // Write zero if underlying page failed to read or index straddling 4K boundary
                            let index_offset = (index_address - virtual_address) as usize;
                            indices[j] = temp
                                .and_then(|temp| temp.get(index_offset..index_offset + 4))
                                .map(|dword| {
                                    u32::from_ne_bytes([dword[0], dword[1], dword[2], dword[3]])
                                })
                                .unwrap_or(0);
                        }
                    }
                }
            }
        }

        if success {
            0
        } else {
            -1
        }
    }

    fn write_memory(&mut self, address: u64, src: &[u8]) -> i32 {
        let Ok(vp) = &self.vp else {
            return -1;
        };
        match vp.mem_write(address, &src.to_vec()) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }

    // Overlay rendering currently not implemented!
    fn r_begin(&mut self, screen: &mut [i32; 2]) -> bool {
        true
    }

    fn r_rect(&mut self, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
        let fill = vgc::sRGBA::unpack(fill);
        let stroke = vgc::sRGBA::unpack(stroke);
        // println!("{:?}", ( x, y, height, width ));
        self.draw_ptr.add(Shape::Rect(RectShape {
            rect: Rect {
                min: Pos2 { x, y },
                max: Pos2 {
                    x: x + width,
                    y: y + height,
                },
            },
            rounding: Rounding::none(),
            fill: Color32::from_rgba_unmultiplied(fill.red, fill.green, fill.blue, fill.alpha),
            stroke: Stroke::new(
                2.0,
                Color32::from_rgba_unmultiplied(
                    stroke.red,
                    stroke.green,
                    stroke.blue,
                    stroke.alpha,
                ),
            ),
        }));
    }

    fn r_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
        let fill = vgc::sRGBA::unpack(fill);
        let stroke = vgc::sRGBA::unpack(stroke);

        // println!("{:?}", ( x, y, height, width ));
        self.draw_ptr.add(Shape::Circle(CircleShape {
            center: Pos2 { x, y },
            radius: 5.0,
            fill: Color32::from_rgba_unmultiplied(fill.red, fill.green, fill.blue, fill.alpha),
            stroke: Stroke {
                width: 5.0,
                color: Color32::from_rgba_unmultiplied(
                    stroke.red,
                    stroke.green,
                    stroke.blue,
                    stroke.alpha,
                ),
            },
        }));
    }
    fn r_text(
        &mut self,
        font: u32,
        flags: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
        color2: u32,
        text: &str,
    ) {
        let color = vgc::sRGBA::unpack(color);
        let color2 = vgc::sRGBA::unpack(color2);
        self.draw_ptr.text(
            Pos2 { x: x - 1f32, y: y + 1f32 },
            Align2::LEFT_TOP,
            text,
            FontId::default(),
            Color32::from_rgba_unmultiplied(color2.red, color2.green, color2.blue, color2.alpha),
        );
        self.draw_ptr.text(
            Pos2 { x, y },
            Align2::LEFT_TOP,
            text,
            FontId::default(),
            Color32::from_rgba_unmultiplied(color.red, color.green, color.blue, color.alpha),
        );
    }
    fn r_line(&mut self, color: u32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let color = vgc::sRGBA::unpack(color);

        self.draw_ptr.add(Shape::line_segment(
            [Pos2 { x: x1, y: y1 }, Pos2 { x: x2, y: y2 }],
            Stroke::new(
                2.0,
                Color32::from_rgba_unmultiplied(color.red, color.green, color.blue, color.alpha),
            ),
        ));
    }

    fn r_lines(&mut self, color: u32, points: &[[f32; 2]], lines: &[[u16; 2]]) {
        let color = vgc::sRGBA::unpack(color);
        let mut shapes: Vec<Shape> = vec![];
        for line in lines {
            shapes.push(Shape::line_segment(
                [
                    Pos2::from(points[line[0] as usize]),
                    Pos2::from(points[line[1] as usize]),
                ],
                Stroke {
                    width: 2.0,
                    color: Color32::from_rgba_unmultiplied(
                        color.red,
                        color.green,
                        color.blue,
                        color.alpha,
                    ),
                },
            ))
        }
        self.draw_ptr.extend(shapes);
    }

    fn r_image(
        &mut self,
        image: u32,
        sx: f32,
        sy: f32,
        swidth: f32,
        sheight: f32,
        dx: f32,
        dy: f32,
        dwidth: f32,
        dheight: f32,
        opacity: f32,
    ) {}

    fn r_end(&mut self) {
        self.context.request_repaint()
    }
}
