use esp_idf_svc::sys::link_patches;

fn main() {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("=============================================");
    println!("        K10 Modbus/TCP Server");
    println!("=============================================");
    println!("Configure Wi-Fi and Modbus context here.");
}
