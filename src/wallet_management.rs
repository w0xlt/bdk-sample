use bdk::bitcoincore_rpc::{Client, Auth as core_rpc_auth, RpcApi};

pub fn load_wallet(wallet_name: String) {
    println!("wallet name");

    let rpc_auth = core_rpc_auth::UserPass(
        "admin".to_string(),
        "password".to_string()
    );

    let mut url = "http://127.0.0.1:38332/wallet/".to_string();
    url.push_str(&wallet_name);

    let core_rpc = Client::new(&url, rpc_auth).unwrap();
    // println!("{:#?}", core_rpc.load_wallet(&wallet_name).unwrap());

    let load_result = core_rpc.load_wallet(&wallet_name);

    match load_result {
        Ok(_) => println!("wallet {:?} loaded successfully", &wallet_name),
        Err(error) => println!("Problem getting blockchain info: {:?}", error),
    }

    println!("{:#?}", core_rpc.get_balance(Some(0), Some(true)).unwrap());
    // println!("{:#?}", core_rpc.unload_wallet(Some(&wallet_name)).unwrap());
}