use std::sync::Once;
use std::thread::sleep;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use egui_backend::egui::{LayerId, Painter};
use egui_backend::WindowBackend;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui::epaint::{CircleShape, RectShape, TextShape};
use egui_overlay::egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Shape, Stroke};

use egui_overlay::EguiOverlay;
use crate::SendObject;
use crate::Flags;
use crate::math;
use crate::*;
fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "Noto_Sans_SC".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../fonts/Noto_Sans_SC/NotoSansSC-VariableFont_wght.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Noto_Sans_SC".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("Noto_Sans_SC".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn get_context(sx: Sender<egui_backend::egui::Context>, rx: Receiver<(Vec<SendObject>, [f32; 16], [i32; 2], [f32; 3])>) {
    egui_overlay::start(DrawFrame { sx, rx, objects: Vec::new(), vmatrix: [0f32; 16], screen: [0i32; 2], camera_origin: [0f32; 3]});
}


pub struct DrawFrame {
    pub sx: Sender<egui_backend::egui::Context>,
    pub rx: Receiver<(Vec<SendObject>, [f32; 16], [i32; 2], [f32; 3])>,
    pub objects: Vec<SendObject>,
    pub vmatrix: [f32; 16],
    pub screen: [i32; 2],
    pub camera_origin: [f32; 3],
}

impl EguiOverlay for crate::overlay::DrawFrame {
    fn gui_run(
        &mut self,
        egui_context: &egui_backend::egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            glfw_backend.set_window_position([0f32; 2]);
            glfw_backend.set_window_size([2560.0f32 - 1.0, 1440.0f32 - 1.0]);
            self.sx.send(egui_context.clone()).unwrap();
        });

        match self.rx.try_recv() {
            Ok((object, vmatrix, screen, camera_origin)) => {
                self.objects = object;
                self.vmatrix = vmatrix;
                self.screen = screen;
                self.camera_origin = camera_origin;
            }
            Err(_) => ()
        };
        let mut ptr = Painter::new(egui_context.clone(), LayerId::debug(), Rect::EVERYTHING);
        for object in &self.objects {
            draw_send_object(&mut ptr, &object, &self.vmatrix, &self.screen, &self.camera_origin);
        }
        // here you decide if you want to be passthrough or not.
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.window.set_mouse_passthrough(false);
        } else {
            glfw_backend.window.set_mouse_passthrough(true);
        }
        // sleep(Duration::from_millis(30));
        egui_context.request_repaint();
    }
}
#[derive(Copy, Clone, Debug)]
pub struct ObjectBounds {
    pub cx: f32,
    pub cy: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
fn r_rect(ptr: &mut Painter, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
    let fill = vgc::sRGBA::unpack(fill);
    let stroke = vgc::sRGBA::unpack(stroke);
    // println!("{:?}", ( x, y, height, width ));
    ptr.add(Shape::Rect(RectShape {
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

fn r_ellipse(ptr: &mut Painter, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
    let fill = vgc::sRGBA::unpack(fill);
    let stroke = vgc::sRGBA::unpack(stroke);

    // println!("{:?}", ( x, y, height, width ));
    ptr.add(Shape::Circle(CircleShape {
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
    ptr: &mut Painter,
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

    let text_rect = |ptr: &egui_overlay::egui::Painter, pos: Pos2, anchor: Align2, text: &str, font_id: FontId, text_color: Color32 | -> Shape {
        let galley = ptr.layout_no_wrap(text.to_string(), font_id, text_color);
        let rect = anchor.anchor_rect(Rect::from_min_size(pos, galley.size()));
        TextShape::new(rect.min, galley).into()
    };

    let text1= text_rect(
        &ptr,
        Pos2 { x: x - 1f32, y: y + 1f32 },
        Align2::LEFT_TOP,
        text,
        FontId::default(),
        Color32::from_rgba_unmultiplied(color2.red, color2.green, color2.blue, color2.alpha),
    );
    let text_shadow = text_rect(
        &ptr,
        Pos2 { x, y },
        Align2::LEFT_TOP,
        text,
        FontId::default(),
        Color32::from_rgba_unmultiplied(color.red, color.green, color.blue, color.alpha),
    );

    ptr.add(text1);
    ptr.add(text_shadow);

}
fn r_line(ptr: &mut Painter, color: u32, x1: f32, y1: f32, x2: f32, y2: f32) {
    let color = vgc::sRGBA::unpack(color);

    ptr.add(Shape::line_segment(
        [Pos2 { x: x1, y: y1 }, Pos2 { x: x2, y: y2 }],
        Stroke::new(
            2.0,
            Color32::from_rgba_unmultiplied(color.red, color.green, color.blue, color.alpha),
        ),
    ));
}

fn r_lines(ptr: &mut Painter, color: u32, points: &[[f32; 2]], lines: &[[u16; 2]]) {
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
    ptr.extend(shapes);
}
#[inline(never)]
fn draw_send_object(ptr: &mut Painter, object: &SendObject, vmatrix: &[f32; 16], screen: &[i32; 2], camera_origin: &[f32; 3]) {
    fn world_to_screen(vmatrix: &[f32; 16], v: [f32; 3], screen: &[i32; 2], clip: bool) -> Option<[f32; 2]> {

        let w = vmatrix[12] * v[0] + vmatrix[13] * v[1] + vmatrix[14] * v[2] + vmatrix[15];
        if w < 0.01 {
            return None;
        }

        let invw = 1.0 / w;
        let vx = (vmatrix[0] * v[0] + vmatrix[1] * v[1] + vmatrix[2] * v[2] + vmatrix[3]) * invw;
        let vy = (vmatrix[4] * v[0] + vmatrix[5] * v[1] + vmatrix[6] * v[2] + vmatrix[7]) * invw;

        // If the resulting coordinate is too far outside the screen bounds clip it manually
        if clip {
            if vx < -2.0 || vx > 2.0 || vy < -2.0 || vy > 2.0 {
                return None;
            }
        }

        let width = screen[0] as f32 * 0.5;
        let height = screen[1] as f32 * 0.5;

        let px = width + vx * width + 0.5;
        let py = height - vy * height + 0.5;
        Some([px, py])
    }
    fn angles_to_screen(camera_origin: &[f32; 3], vmatrix: &[f32; 16], a: [f32; 3], screen: &[i32; 2], clip: bool) -> Option<[f32; 2]> {
        let dir = qvec(a);
        let point = add(*camera_origin, muls(dir, 1000.0));
        world_to_screen(vmatrix, point, screen, clip)
    }

    fn bounds(object: &SendObject, vmatrix: &[f32; 16], screen: &[i32; 2], camera_origin: &[f32; 3]) -> Option<ObjectBounds> {
        // For objects which don't specify their width or height assume a point
        if object.width <= 0.0 || object.height <= 0.0 {
            let [cx, cy] = world_to_screen(vmatrix, object.origin, screen, true)?;
            return Some(ObjectBounds {
                cx,
                cy,
                left: cx,
                top: cy,
                right: cx,
                bottom: cy,
            });
        }

        // This logic is necessary to make the bounds behave nice looking nearly up/down

        let hwidth = object.width * 0.5;

        let dir = math::sub(*camera_origin, object.origin);
        let dir = math::norm([dir[0], dir[1], 0.0]);
        let vbot1 = math::add(object.origin, math::muls(dir, hwidth));
        let vbot2 = math::add(object.origin, math::muls(dir, -hwidth));
        let bot1 = world_to_screen(vmatrix, vbot1, screen, true)?;
        let bot2 = world_to_screen(vmatrix, vbot2, screen, true)?;

        let vtop = [
            object.origin[0],
            object.origin[1],
            object.origin[2] + object.height,
        ];
        let dir = math::sub(*camera_origin, vtop);
        let dir = math::norm([dir[0], dir[1], 0.0]);
        let vtop1 = math::add(vtop, math::muls(dir, hwidth));
        let vtop2 = math::add(vtop, math::muls(dir, -hwidth));
        let top1 = world_to_screen(vmatrix, vtop1, screen, true)?;
        let top2 = world_to_screen(vmatrix, vtop2, screen, true)?;

        let y1 = bot1[1].min(bot2[1]).min(top1[1]).min(top2[1]);
        let y2 = bot1[1].max(bot2[1]).max(top1[1]).max(top2[1]);
        let ph = y2 - y1;
        let pw = ph * (object.width / object.height);

        let [cx, cy] = world_to_screen(vmatrix,
                                       [
                                           object.origin[0],
                                           object.origin[1],
                                           object.origin[2] + object.height * 0.5,
                                       ],screen,
                                       true,
        )?;
        let width = pw * 0.5 * 1.5 + 5.0;
        let height = ph * 0.5 * 1.5 + 5.0;
        Some(ObjectBounds {
            cx,
            cy,
            left: cx - width,
            right: cx + width,
            top: cy - height,
            bottom: cy + height,
        })
    }
    fn alpha(dist: f32, max: f32, factor: f32) -> Option<f32> {
        if dist >= max {
            return None;
        }
        let a = f32::atan((1.0 - dist / max) * factor) / f32::atan(factor);
        let b = a.min(1.0).max(0.0);
        return Some(b);
    }

    let alpha = if object.flags & Flags::ALPHA {
        object.alpha
    } else {
        match alpha(object.distance, object.fade_dist, 1.0) {
            Some(a) => a,
            None => return,
        }
    };
    let alpha = (alpha * 255.0) as u8;
    let color = object.color.alpha(alpha).into();
    let light = vgc::sRGBA(255, 255, 255, alpha).into();
    let dark = vgc::sRGBA(0, 0, 0, alpha / 2 + alpha / 4).into();
    let nearby = math::dist2(*camera_origin, object.origin) < 2048.0 * 2048.0;



    if object.skynade_pitch != 0.0 {
        // let delta = sub(target, local.view_origin);
        // let dx = f32::sqrt(delta[0] * delta[0] + delta[1] * delta[1]);
        // let dir = [delta[0] / dx, delta[1] / dx, pitch.tan()];
        // let point = add(mul(dir, [1000.0; 3]), local.view_origin);

        if let Some([px, py]) = angles_to_screen(
            camera_origin,
            vmatrix,
            [
                -object.skynade_pitch.to_degrees(),
                object.skynade_yaw.to_degrees(),
                0.0,
            ],
            screen,
            false,
        ) {
            let px = px.round();
            let py = py.round();
            if let Some([sx, sy]) = world_to_screen(vmatrix, object.origin, screen, false) {
                r_line(ptr,
                    /*color:*/ vgc::sRGBA!(White).into(),
                    /*x1:*/ sx.round(),
                    /*y1:*/ sy.round(),
                    /*x2:*/ px,
                    /*y2:*/ py,
                );
            }
            r_ellipse(ptr,
                /*x:*/ px - 3.0,
                /*y:*/ py - 3.0,
                /*width:*/ 6.0,
                /*height:*/ 6.0,
                /*fill:*/ vgc::sRGBA(0xff, 0, 0x66, 255).into(),
                /*stroke:*/ vgc::sRGBA!(Black).into(),
            );
            r_text(ptr,
                /*font:*/ 0,
                /*flags:*/ 3,
                px,
                py,
                /*width:*/ 1000.0,
                /*height:*/ 100.0,
                /*color*/ vgc::sRGBA!(White).into(),
                /*shadow:*/ vgc::sRGBA!(Black).into(),
                /*text:*/ &fmtools::format!({object.skynade_pitch.to_degrees():.1}"°"),
            );
        }
    }


    if object.flags & Flags::BOX {
        let hwidth = object.width * 0.5;
        let compute_pts = || {
            Some([
                world_to_screen(vmatrix, math::add(object.origin, [-hwidth, -hwidth, 0.0]), screen, true)?,
                world_to_screen(vmatrix, math::add(object.origin, [-hwidth, hwidth, 0.0]), screen, true)?,
                world_to_screen(vmatrix, math::add(object.origin, [hwidth, hwidth, 0.0]), screen, true)?,
                world_to_screen(vmatrix, math::add(object.origin, [hwidth, -hwidth, 0.0]), screen, true)?,
                world_to_screen(vmatrix, math::add(object.origin, [-hwidth, -hwidth, object.height]), screen,true,)?,
                world_to_screen(vmatrix, math::add(object.origin, [-hwidth, hwidth, object.height]), screen,true,)?,
                world_to_screen(vmatrix, math::add(object.origin, [hwidth, hwidth, object.height]), screen,true,)?,
                world_to_screen(vmatrix, math::add(object.origin, [hwidth, -hwidth, object.height]), screen,true,)?,
            ])
        };
        if let Some(pts) = compute_pts() {
            static LINES: [[u16; 2]; 12] = [
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 0],
                [4, 5],
                [5, 6],
                [6, 7],
                [7, 4],
                [0, 4],
                [1, 5],
                [2, 6],
                [3, 7],
            ];
            r_lines(ptr, color, &pts, &LINES);
        }
    }

    if object.flags & Flags::ORIGIN {
        if let Some([x, y]) = world_to_screen(vmatrix, object.origin, screen, true) {
            r_rect(ptr,
                /*x:*/ x - 2.0,
                /*y:*/ y - 2.0,
                /*width:*/ 4.0,
                /*height:*/ 4.0,
                /*fill:*/ color,
                /*stroke:*/ dark,
            );
        }
    }

    if object.flags & Flags::SPINE && nearby {
        let p1 = math::add(object.origin, object.spine[0]);
        let p2 = math::add(object.origin, object.spine[1]);

        if let Some([x1, y1]) = world_to_screen(vmatrix,p1, screen,true) {
            if let Some([x2, y2]) = world_to_screen(vmatrix,p2, screen,true) {
                r_line(ptr,color, x1, y1, x2, y2);
            }
        }
    }

    if object.flags & Flags::BARREL {
        let p1 = math::add(object.origin, object.spine[0]);
        let p2 = math::add(p1, math::muls(object.view, 40.0));

        if let Some([x1, y1]) = world_to_screen(vmatrix,p1, screen,true) {
            if let Some([x2, y2]) = world_to_screen(vmatrix,p2, screen,true) {
                r_line(ptr,color, x1, y1, x2, y2);
            }
        }
    }

    if (object.flags & Flags::AIM) && object.visible {
        if let Some(aim) = object.aim {
            if let Some([x, y]) = angles_to_screen(camera_origin, vmatrix, aim, screen, true) {
                let x = x.round();
                let y = y.round();
                r_rect(ptr,
                    /*x:*/ x - 4.0,
                    /*y:*/ y - 1.0,
                    /*width:*/ 9.0,
                    /*height:*/ 3.0,
                    /*fill:*/ dark,
                    /*stroke:*/ vgc::sRGBA::TRANSPARENT.into(),
                );
                r_rect(ptr,
                    /*x:*/ x - 1.0,
                    /*y:*/ y - 4.0,
                    /*width:*/ 3.0,
                    /*height:*/ 9.0,
                    /*fill:*/ dark,
                    /*stroke:*/ vgc::sRGBA::TRANSPARENT.into(),
                );
                r_line(ptr,color, x - 3.0, y, x + 3.0, y);
                r_line(ptr,color, x, y - 3.0, x, y + 3.0);
            }
        }
    }

    let Some(bounds) = bounds(object, vmatrix, screen, camera_origin) else {
        return;
    };

    if object.flags & Flags::TEXT {
        if let Some(text) = &object.text {
            if let Some([x, y]) = world_to_screen( vmatrix, object.origin, screen, true) {
                r_text(ptr,
                    /*font:*/ 0, /*flags:*/ 3, x, y, /*width:*/ 1000.0,
                    /*height:*/ 100.0, color, /*shadow:*/ dark, text.as_str(),
                );
            }
        }
    }

    if object.flags & Flags::NAME {
        if let Some(name) = &object.name {
            if let Some([x, y]) = world_to_screen(vmatrix, object.origin, screen, true) {
                r_text(ptr,
                    /*font:*/ 0, /*flags:*/ 3, x, y, /*width:*/ 1000.0,
                    /*height:*/ 100.0, color, /*shadow:*/ dark, /*text:*/ name.as_str(),
                );
            }
        }
    }

    if object.flags & Flags::HEALTH {
        let width = bounds.right - bounds.left;
        if width >= 12.0 {
            let height = 3.0;
            let x = bounds.left;
            let y = bounds.bottom + 2.0;
            r_rect(ptr,
                /*x:*/ x - 0.5,
                /*y:*/ y - 0.5,
                /*width:*/ width + 1.0,
                /*height:*/ height + 1.0,
                /*fill:*/ dark,
                /*stroke:*/ vgc::sRGBA::TRANSPARENT.into(),
            );
            if object.max_health > 0 {
                let health = f32::min(1.0, object.health as f32 / object.max_health as f32);
                let width = width * health;
                r_rect(ptr,
                    x,
                    y,
                    width,
                    height,
                    /*fill:*/ light,
                    /*stroke:*/ vgc::sRGBA::TRANSPARENT.into(),
                );
            }
            if object.max_shields > 0 {
                let shields = f32::min(1.0, object.shields as f32 / object.max_shields as f32);
                let width = width * shields;
                r_rect(ptr,
                    /*x:*/ x + 0.5,
                    /*y:*/ y + 0.5,
                    /*width:*/ width - 1.0,
                    /*height:*/ height - 1.0,
                    /*fill:*/ color,
                    /*stroke:*/ vgc::sRGBA::TRANSPARENT.into(),
                );
            }
        }
    }



    if object.flags & Flags::BOUNDS {
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        if object.visible {
            r_rect(ptr,
                /*x:*/ bounds.left + 1.0,
                /*y:*/ bounds.top + 1.0,
                width,
                height,
                /*fill:*/ vgc::sRGBA::TRANSPARENT.into(),
                /*stroke:*/ dark,
            );
            r_rect(ptr,
                /*x:*/ bounds.left,
                /*y:*/ bounds.top,
                width,
                height,
                /*fill:*/ vgc::sRGBA::TRANSPARENT.into(),
                /*stroke:*/ color,
            );
            r_text(ptr,
                /*font:*/ 0, /*flags:*/ 3, /*x:*/ bounds.left,
                /*y:*/ bounds.top, /*width:*/ 1000.0,
                /*height:*/ 100.0, color, /*shadow:*/ dark, /*text:*/ "vis",
            );
        } else {
            let size = f32::min(width, height) / 4.0;
            let width = width - 1.0;
            let height = height - 1.0;
            let x = bounds.left + 1.0;
            let y = bounds.top + 1.0;
            let mut points = [
                [x, y],
                [x, y + size],
                [x + size, y],
                [x + width, y],
                [x + width, y + size],
                [x + width - size, y],
                [x, y + height],
                [x, y + height - size],
                [x + size, y + height],
                [x + width, y + height],
                [x + width, y + height - size],
                [x + width - size, y + height],
            ];
            static LINES: [[u16; 2]; 8] = [
                [0, 1],
                [0, 2],
                [3, 4],
                [3, 5],
                [6, 7],
                [6, 8],
                [9, 10],
                [9, 11],
            ];
            r_lines(ptr,
                /*color:*/ dark, /*points:*/ &points, /*lines:*/ &LINES,
            );
            for p in &mut points {
                p[0] -= 1.0;
                p[1] -= 1.0;
            }
            r_lines(ptr, color, /*points:*/ &points, /*lines:*/ &LINES);
        }
    }
}