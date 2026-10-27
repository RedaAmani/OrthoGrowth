// Dessine l'interface dans des images PPM (sans écran), pour vérifier la mise en page.
use orthogrowth_app::{build, Form};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::io::Write;
use std::rc::Rc;

struct P(Rc<MinimalSoftwareWindow>);
impl Platform for P {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> { Ok(self.0.clone()) }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "/tmp/shots".into());
    std::fs::create_dir_all(&out).unwrap();
    let w: u32 = std::env::var("SHOT_W").ok().and_then(|v| v.parse().ok()).unwrap_or(1100);
    let h: u32 = std::env::var("SHOT_H").ok().and_then(|v| v.parse().ok()).unwrap_or(900);
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(P(window.clone()))).unwrap();
    let data = std::env::temp_dir().join(format!("og-shot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let ui = build(data.join("donnees"));
    window.set_size(PhysicalSize::new(w, h));
    if std::env::var("SHOT_MOBILE").is_ok() { ui.set_mobile(true); }
    ui.show().unwrap();
    let shot = |name: &str| {
        slint::platform::update_timers_and_animations();
        let (w, h) = (window.size().width, window.size().height);
        let mut buf = SharedPixelBuffer::<Rgb8Pixel>::new(w, h);
        window.draw_if_needed(|r| { r.render(buf.make_mut_slice(), w as usize); });
        let mut f = std::fs::File::create(format!("{out}/{name}.ppm")).unwrap();
        write!(f, "P6\n{w} {h}\n255\n").unwrap();
        for p in buf.as_slice() { f.write_all(&[p.r, p.g, p.b]).unwrap(); }
    };
    shot("01-vide-fr");
    ui.set_about_open(true);
    shot("01b-apropos-fr");
    ui.set_about_open(false);
    let mut f: Form = ui.get_form();
    f.last_name = "Test".into(); f.first_names = "Fictif".into(); f.age = "10".into();
    f.femur_r = "400".into(); f.femur_l = "390".into(); f.tibia_r = "330".into(); f.tibia_l = "325".into();
    ui.set_form(f);
    ui.invoke_calculate();
    window.set_size(PhysicalSize::new(w, 2300));
    shot("02-calcul-fr");
    window.set_size(PhysicalSize::new(w, h));
    ui.invoke_save_record();
    ui.set_tab(1);
    shot("03-dossiers-fr");
    ui.set_tab(2);
    shot("04-table-fr");
    ui.set_tab(0);
    let mut f: Form = ui.get_form();
    f.scenario = 1; f.prev_femur_r = "350".into(); f.prev_femur_l = "345".into(); f.prev_tibia_r = "290".into(); f.prev_tibia_l = "287".into();
    f.prev_age = "7".into();
    ui.set_form(f);
    ui.invoke_calculate();
    ui.invoke_lang_changed(1); ui.set_lang_index(1);
    window.set_size(PhysicalSize::new(w, 2500));
    shot("05-developpemental-en");
    window.set_size(PhysicalSize::new(w, h));
    ui.set_about_open(true);
    shot("06-apropos-en");
}
