use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::time::{Duration, Instant};

use escapi::{Device, Error};
use pixels::{Pixels, SurfaceTexture, wgpu};
use pixels::wgpu::Backend;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

// For now up -> down only.
struct Line {
    frame_width: usize,
    frame_height: usize,
    y_pos: usize,
    size: usize,
    currently_frozen: Vec<u8>,
}

impl Line {
    fn new() -> Self {
        Self {
            frame_width: 640,
            frame_height: 480,
            y_pos: 0,
            size: 1,
            currently_frozen: vec![],
        }
    }
}

trait FrameSource {
    fn next_frame(&mut self) -> Option<Vec<u8>>;
}

struct LineDrawingDevice {
    device: Device,
    line: Line,
}

impl FrameSource for LineDrawingDevice {
    fn next_frame(&mut self) -> Option<Vec<u8>> {
        let frame = self.device.next_frame()?;
        let width_4 = self.line.frame_width * 4;
        let yank_from = self.line.y_pos * width_4;
        let to_yank = self.line.size * width_4;
        let yank_to = yank_from + to_yank;

        self.line.currently_frozen.extend_from_slice(&frame.as_slice()[yank_from..yank_to]);
        self.line.y_pos += 1;
        let result_frame = merge(frame, &self.line);
        result_frame
    }
}

fn merge(frame: Vec<u8>, line: &Line) -> Option<Vec<u8>> {
    let width_4 = line.frame_width * 4;
    let max_len = line.frame_height * width_4;
    let frozen_part = &line.currently_frozen;
    return if frozen_part.len() >= max_len {
        None
    } else {
        // (163, 73, 164, 0) is purple.
        let mut purple_line: Vec<u8> = vec!();
        for _ in 0..line.frame_width * 4 {
            purple_line.push(163);
            purple_line.push(73);
            purple_line.push(164);
            purple_line.push(126);
        }
        let mut result = vec!();
        result.extend_from_slice(frozen_part);
        result.extend_from_slice(&purple_line.as_slice());
        if result.len() >= max_len {
            None
        } else {
            let new_part = &frame[result.len()..max_len];
            result.extend_from_slice(new_part);
            Some(result)
        }
    }
}

impl FrameSource for Device {
    fn next_frame(&mut self) -> Option<Vec<u8>> {
        // TODO: deal with `unwrap` later
        // TODO: clone not so good
        let mut frame = self.capture().unwrap().to_owned();
        to_rgba(&mut frame);
        Some(frame)
    }
}

fn to_rgba(windows_specific_pixels: &mut Vec<u8>) {
    for chunk in windows_specific_pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2)
    }
}

fn main() {
    let mut device = escapi::init(0, 640, 480, 60)
        .unwrap();
    let (window, event_loop) = init_window();
    let line_device = LineDrawingDevice {
        device,
        line: Line::new(),
    };
    let now = Instant::now();
    start_drawing(window, event_loop, line_device);
    let after = Instant::now();
    println!("the whole thing took {} seconds", after.duration_since(now).as_secs());
}

fn start_drawing<F: 'static + FrameSource>(window: Window, event_loop: EventLoop<()>, mut frame_source: F) {
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(window_size.width, window_size.height, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::RedrawRequested(w_id) => {
                if w_id == window.id() {
                    let next_frame = frame_source.next_frame();
                    if next_frame.is_none() {
                        *control_flow = ControlFlow::Exit;
                    } else {
                        pixels.get_frame().copy_from_slice(next_frame.unwrap().as_slice());
                        pixels.render();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
        window.request_redraw();
    });
}

fn init_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(640, 480);
    (WindowBuilder::new()
         .with_inner_size(size)
         .build(&event_loop).unwrap(), event_loop)
}
