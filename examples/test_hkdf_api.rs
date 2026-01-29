use keyring_cli::crypto::derive_device_key;

fn main() {
    let master_key = [0u8; 32];
    let device_id = "test-device-123";
    
    let device_key = derive_device_key(&master_key, device_id);
    
    println!("Device ID: {}", device_id);
    println!("Device Key (hex): {:02x}", device_key[0]);
    assert_eq!(device_key.len(), 32);
    println!("API test passed!");
}
