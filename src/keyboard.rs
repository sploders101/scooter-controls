use esp32_nimble::{
    enums::*, hid::*, utilities::mutex::Mutex, BLECharacteristic, BLEDevice, BLEHIDDevice,
    BLEServer,
};
use esp_idf_sys as _;
use std::sync::Arc;

const MEDIA_KEYS_ID: u8 = 0x01;
const HID_REPORT_DESCRIPTOR: &'static [u8] = hid!(
    (USAGE_PAGE, 0x0C),         // USAGE_PAGE (Consumer)
    (USAGE, 0x01),              // USAGE (Consumer Control)
    (COLLECTION, 0x01),         // COLLECTION (Application)
    (REPORT_ID, MEDIA_KEYS_ID), //   REPORT_ID (3)
    (USAGE_PAGE, 0x0C),         //   USAGE_PAGE (Consumer)
    (LOGICAL_MINIMUM, 0x00),    //   LOGICAL_MINIMUM (0)
    (LOGICAL_MAXIMUM, 0x01),    //   LOGICAL_MAXIMUM (1)
    (REPORT_SIZE, 0x01),        //   REPORT_SIZE (1)
    (REPORT_COUNT, 0x08),       //   REPORT_COUNT (8)
    (USAGE, 0xB5),              //   USAGE (Scan Next Track)     ; bit 0: 1
    (USAGE, 0xB6),              //   USAGE (Scan Previous Track) ; bit 1: 2
    (USAGE, 0xB7),              //   USAGE (Stop)                ; bit 2: 4
    (USAGE, 0xCD),              //   USAGE (Play/Pause)          ; bit 3: 8
    (USAGE, 0xE2),              //   USAGE (Mute)                ; bit 4: 16
    (USAGE, 0xE9),              //   USAGE (Volume Increment)    ; bit 5: 32
    (USAGE, 0xEA),              //   USAGE (Volume Decrement)    ; bit 6: 64
    (USAGE, 0xCF),              //   USAGE (Siri)                ; bit 7: 128
    (HIDINPUT, 0x02), //   INPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (END_COLLECTION)  // END_COLLECTION
);

#[allow(dead_code)]
#[repr(C)]
struct KeyReport {
    modifiers: u8,
    reserved: u8,
    keys: [u8; 6],
}

pub struct Keyboard {
    device: &'static mut BLEDevice,
    server: &'static mut BLEServer,
    input_media_keys: Arc<Mutex<BLECharacteristic>>,
}

impl Keyboard {
    pub fn new() -> Self {
        BLEDevice::set_device_name("Rusty Scooter").unwrap();
        let device = BLEDevice::take();
        device
            .security()
            .set_auth(AuthReq::all())
            .set_io_cap(SecurityIOCap::NoInputNoOutput);

        let server = device.get_server();
        let mut hid = BLEHIDDevice::new(server);

        let input_media_keys = hid.input_report(MEDIA_KEYS_ID);

        hid.manufacturer("Shaun Keys");
        hid.pnp(0x02, 0x05ac, 0x820a, 0x0210);
        hid.hid_info(0x00, 0x01);

        hid.report_map(&HID_REPORT_DESCRIPTOR);

        hid.set_battery_level(100);

        let ble_advertising = device.get_advertising();
        ble_advertising
            .name("Rusty Scooter")
            .appearance(0x03C1)
            .add_service_uuid(hid.hid_service().lock().uuid())
            .scan_response(false);
        ble_advertising.start().unwrap();

        Self {
            device,
            server,
            input_media_keys,
        }
    }

    pub fn advertise(&self) {
        self.device.get_advertising().start().unwrap();
    }

    pub fn connected(&self) -> bool {
        self.server.connected_count() > 0
    }

    pub fn send_report<T: Sized>(&self, keys: &T) {
        self.input_media_keys.lock().set_from(keys).notify();
        esp_idf_hal::delay::Ets::delay_ms(7);
    }

    pub fn send_siri(&self) {
        self.send_report(&[128, 0]);
        esp_idf_hal::delay::FreeRtos::delay_ms(500);
        self.send_report(&[0, 0]);
    }

    pub fn play_pause(&self) {
        self.send_report(&[8, 0]);
        self.send_report(&[0, 0]);
    }

    pub fn skip(&self) {
        self.send_report(&[1, 0]);
        self.send_report(&[0, 0]);
    }

    pub fn back(&self) {
        self.send_report(&[2, 0]);
        self.send_report(&[0, 0]);
    }

    pub fn volume_up(&self) {
        self.send_report(&[32, 0]);
        self.send_report(&[0, 0]);
    }

    pub fn volume_down(&self) {
        self.send_report(&[64, 0]);
        self.send_report(&[0, 0]);
    }
}
