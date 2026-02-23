use serde::{Deserialize, Serialize};
use serialport::{SerialPortType, available_ports};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PortType {
    key: String,
    value: String,
}

pub fn get_list_serial_ports() -> String {
    if let Ok(ports) = available_ports() {
        let mut result: Vec<PortType> = Vec::new(); 
        for port in ports {
            match port.port_type {
                SerialPortType::UsbPort(info) => {
                    let port_info = PortType{
                        key: format!("Port: {} {} {}", port.port_name, info.product.unwrap_or(String::from("Unknown")), info.manufacturer.unwrap_or(String::from("Unknown"))),
                        value: port.port_name,
                    };

                    result.push(port_info);
                }
                SerialPortType::PciPort => {},
                SerialPortType::BluetoothPort => {},
                SerialPortType::Unknown => {},
            }
        }

        match serde_json::to_string(&result) {
            Ok(json_str) => return json_str,
            Err(_) => return "[]".into()
        }
    }

    return "[]".into();
}