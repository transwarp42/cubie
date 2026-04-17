/// Pixel dimensions of the generated icon bitmap.
const ICON_PIXELS: u32 = 256;
/// Display size in points (macOS retina: 256px at 128pt = 2x).
#[cfg(target_os = "macos")]
const ICON_POINTS: f64 = 128.0;

// Isometric cube geometry
const EDGE: f32 = 100.0;
const GAP_BORDER: f32 = 0.042;
const GAP_INNER: f32 = 0.022;
const BODY_COLOR: [u8; 4] = [22, 22, 22, 255];

struct Face {
    origin: [f32; 2],
    u: [f32; 2],
    v: [f32; 2],
    color: [f32; 3],
    brightness: f32,
}

/// Sample a single sub-pixel and return its RGBA color.
fn sample_at(p: [f32; 2], faces: &[Face; 3]) -> [u8; 4] {
    for face in faces {
        let dp = [p[0] - face.origin[0], p[1] - face.origin[1]];
        let cross = face.u[0] * face.v[1] - face.u[1] * face.v[0];
        if cross.abs() < 0.001 {
            continue;
        }

        let s = (dp[0] * face.v[1] - dp[1] * face.v[0]) / cross;
        let t = (face.u[0] * dp[1] - face.u[1] * dp[0]) / cross;

        if s < 0.0 || s > 1.0 || t < 0.0 || t > 1.0 {
            continue;
        }

        // Determine which of the 3×3 cells we're in
        let ci = (s * 3.0).floor().min(2.0) as usize;
        let cj = (t * 3.0).floor().min(2.0) as usize;

        // Sticker boundaries (thicker border at face edges, thinner between stickers)
        let s0 = ci as f32 / 3.0 + if ci == 0 { GAP_BORDER } else { GAP_INNER };
        let s1 = (ci + 1) as f32 / 3.0 - if ci == 2 { GAP_BORDER } else { GAP_INNER };
        let t0 = cj as f32 / 3.0 + if cj == 0 { GAP_BORDER } else { GAP_INNER };
        let t1 = (cj + 1) as f32 / 3.0 - if cj == 2 { GAP_BORDER } else { GAP_INNER };

        if s < s0 || s > s1 || t < t0 || t > t1 {
            return BODY_COLOR;
        }

        // Sticker color with face brightness
        let b = face.brightness;
        return [
            (face.color[0] * b) as u8,
            (face.color[1] * b) as u8,
            (face.color[2] * b) as u8,
            255,
        ];
    }

    // Transparent background
    [0, 0, 0, 0]
}

/// Generate a 256×256 RGBA pixel buffer of an isometric 3D Rubik's Cube.
fn generate_cube_icon() -> Vec<u8> {
    let s = ICON_PIXELS as usize;
    let mut pixels = vec![0u8; s * s * 4];

    let cx = s as f32 / 2.0;
    let cy = s as f32 / 2.0 + 4.0;
    let cos30 = 3.0f32.sqrt() / 2.0;
    let h = EDGE * cos30;
    let v = EDGE * 0.5;

    // Seven visible vertices of the isometric cube hexagon
    let top = [cx, cy - EDGE];
    let tr = [cx + h, cy - v];
    let br = [cx + h, cy + v];
    let _bot = [cx, cy + EDGE];
    let bl = [cx - h, cy + v];
    let tl = [cx - h, cy - v];
    let cen = [cx, cy];

    // Three visible faces: top (white), right (red), left (green)
    let faces: [Face; 3] = [
        Face {
            origin: top,
            u: [tr[0] - top[0], tr[1] - top[1]],
            v: [tl[0] - top[0], tl[1] - top[1]],
            color: [240.0, 240.0, 240.0],
            brightness: 1.0,
        },
        Face {
            origin: tr,
            u: [br[0] - tr[0], br[1] - tr[1]],
            v: [cen[0] - tr[0], cen[1] - tr[1]],
            color: [210.0, 12.0, 12.0],
            brightness: 0.78,
        },
        Face {
            origin: tl,
            u: [cen[0] - tl[0], cen[1] - tl[1]],
            v: [bl[0] - tl[0], bl[1] - tl[1]],
            color: [0.0, 155.0, 30.0],
            brightness: 0.58,
        },
    ];

    // 2×2 super-sampling for anti-aliased edges
    for py in 0..s {
        for px in 0..s {
            let idx = (py * s + px) * 4;
            let mut ra = 0u32;
            let mut ga = 0u32;
            let mut ba = 0u32;
            let mut aa = 0u32;

            for sy in 0..2u32 {
                for sx in 0..2u32 {
                    let p = [
                        px as f32 + 0.25 + sx as f32 * 0.5,
                        py as f32 + 0.25 + sy as f32 * 0.5,
                    ];
                    let c = sample_at(p, &faces);
                    ra += c[0] as u32;
                    ga += c[1] as u32;
                    ba += c[2] as u32;
                    aa += c[3] as u32;
                }
            }

            pixels[idx] = (ra / 4) as u8;
            pixels[idx + 1] = (ga / 4) as u8;
            pixels[idx + 2] = (ba / 4) as u8;
            pixels[idx + 3] = (aa / 4) as u8;
        }
    }

    pixels
}

/// Startup system that sets the application icon.
/// On macOS this sets the dock icon via Cocoa APIs.
/// On other platforms this sets the window icon via winit.
pub fn set_app_icon(
    #[cfg(not(target_os = "macos"))]
    winit_windows: bevy::prelude::NonSend<bevy::winit::WinitWindows>,
    #[cfg(not(target_os = "macos"))]
    primary_query: bevy::prelude::Query<
        bevy::prelude::Entity,
        bevy::prelude::With<bevy::window::PrimaryWindow>,
    >,
) {
    let rgba = generate_cube_icon();

    #[cfg(target_os = "macos")]
    set_macos_dock_icon(&rgba);

    #[cfg(not(target_os = "macos"))]
    set_winit_window_icon(&rgba, &winit_windows, &primary_query);
}

#[cfg(target_os = "macos")]
fn set_macos_dock_icon(rgba: &[u8]) {
    use objc2::ClassType;
    use objc2_app_kit::{NSApplication, NSBitmapImageRep, NSImage};
    use objc2_foundation::{MainThreadMarker, NSSize, NSString};

    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let color_space = NSString::from_str("NSCalibratedRGBColorSpace");

        let Some(rep) = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            ICON_PIXELS as isize,
            ICON_PIXELS as isize,
            8,
            4,
            true,
            false,
            &color_space,
            (ICON_PIXELS * 4) as isize,
            32,
        ) else {
            return;
        };

        let bitmap_data = rep.bitmapData();
        if !bitmap_data.is_null() {
            std::ptr::copy_nonoverlapping(rgba.as_ptr(), bitmap_data, rgba.len());
        }

        let size = NSSize::new(ICON_POINTS, ICON_POINTS);
        let image = NSImage::initWithSize(NSImage::alloc(), size);
        image.addRepresentation(&rep);

        let app = NSApplication::sharedApplication(mtm);
        app.setApplicationIconImage(Some(&image));
    }
}

/// Set the window icon on non-macOS platforms via winit.
#[cfg(not(target_os = "macos"))]
fn set_winit_window_icon(
    rgba: &[u8],
    winit_windows: &bevy::prelude::NonSend<bevy::winit::WinitWindows>,
    primary_query: &bevy::prelude::Query<
        bevy::prelude::Entity,
        bevy::prelude::With<bevy::window::PrimaryWindow>,
    >,
) {
    use winit::window::Icon;

    let Ok(entity) = primary_query.get_single() else {
        return;
    };
    let Some(window) = winit_windows.get_window(entity) else {
        return;
    };

    if let Ok(icon) = Icon::from_rgba(rgba.to_vec(), ICON_PIXELS, ICON_PIXELS) {
        window.set_window_icon(Some(icon));
    }
}
