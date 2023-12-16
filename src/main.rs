use esp_idf_sys as _;
use keyboard::Keyboard;

mod keyboard;

fn main() {
    esp_idf_sys::link_patches();

    let keyboard = Keyboard::new();

    let mut retry_counter = 0u8;

    loop {
        if keyboard.connected() {
            keyboard.play_pause();
        } else if retry_counter > 2 {
            keyboard.advertise();
        } else {
            retry_counter = retry_counter.saturating_add(1);
        }
        esp_idf_hal::delay::FreeRtos::delay_ms(5000);
    }
}
