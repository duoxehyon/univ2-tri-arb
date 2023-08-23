use ethers::prelude::*;

/// Converts &str to Address.
pub fn address(address: &str) -> Address {
    address.parse::<Address>().unwrap()
}

/// Creates a binding for an ABI.
/// Example: bind("Example", "src/abi/example.json");
pub fn bind(name: &str, abi: &str) {
    let name: String = name.to_string();
    let bindings = Abigen::new(&name, abi).unwrap().generate().unwrap();
    let path: String = format!("src/bindings/{}.rs", name);
    match std::fs::File::create(path.clone()) {
        Ok(_) => {}
        Err(_) => {}
    }
    bindings.write_to_file(&path).unwrap();
}
