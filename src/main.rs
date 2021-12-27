pub mod wallet_management;

use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{DerivationPath, KeySource};
use bdk::bitcoin::Network;
use bdk::bitcoincore_rpc::{Client, Auth as core_rpc_auth, RpcApi};
use bdk::blockchain::{ConfigurableBlockchain, NoopProgress, LogProgress};
use bdk::descriptor::Segwitv0;
use bdk::keys::{GeneratableKey, DescriptorKey, ExtendedKey};
use bdk::{keys::GeneratedKey};
use bdk::keys::bip39::{Mnemonic, WordCount, Language};
use bdk::miniscript;
use bdk::keys::{DerivableKey, DescriptorKey::Secret};



use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};

use bdk::Wallet;
use bdk::wallet::{AddressIndex, signer::SignOptions, wallet_name_from_descriptor};

use bdk::sled::{self, Tree};

use std::str::FromStr;

fn test_bitcoin_rpc_connection() -> bool {
    let rpc_auth = core_rpc_auth::UserPass(
        "admin".to_string(),
        "password".to_string()
    );

    let core_rpc = Client::new("http://127.0.0.1:38332/".into(), rpc_auth).unwrap();

    let info = core_rpc.get_blockchain_info();

    match info {
        Ok(_) => return true,
        Err(error) => println!("Problem getting blockchain info: {:?}", error),
    };

    false
}


fn get_block_count() -> u64 {
    let rpc_auth = core_rpc_auth::UserPass(
        "admin".to_string(),
        "password".to_string()
    );

    let core_rpc = Client::new("http://127.0.0.1:38332/".into(), rpc_auth).unwrap();

    let block_count = core_rpc.get_block_count();

    match block_count {
        Ok(count) => return count,
        Err(error) => panic!("Problem getting blockchain info: {:?}", error),
    };
}

fn create_wallet(network: &Network, receive_desc: &String, change_desc: &String, block_count: u32) -> Wallet<RpcBlockchain, Tree> {

    // Set RPC username, password and url
    let auth = Auth::UserPass {
        username: "admin".to_string(),
        password: "password".to_string()
    };

    let mut rpc_url = "http://".to_string();
    rpc_url.push_str("127.0.0.1:38332");

    // Use deterministic wallet name derived from descriptor
    let wallet_name = wallet_name_from_descriptor(
        receive_desc,
        Some(change_desc),
        *network,
        &Secp256k1::new()
    ).unwrap();

    println!("wallet name: {:?}", wallet_name);

    // Setup the RPC configuration
    let rpc_config = RpcConfig {
        url: rpc_url,
        auth,
        network: *network,
        wallet_name: wallet_name.clone(),
        skip_blocks: Some(block_count)
    };

    // Use the above configuration to create a RPC blockchain backend
    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();

    // Create the datadir to store wallet data
    let mut datadir = dirs_next::home_dir().unwrap();
    datadir.push(".bdk-dir");
    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(wallet_name.clone()).unwrap();

    // Combine everything and finally create the BDK wallet structure
    let wallet = Wallet::new(receive_desc, Some(change_desc), *network, db_tree, blockchain).unwrap();

    // Sync the wallet
    //let sync_result = wallet.sync(LogProgress, None);
    wallet.sync(LogProgress, None).unwrap();


    /*match sync_result {
        Ok(_) => println!("Synchronization OK."),
        Err(error) => println!("Problem synchronizing wallet: {:?}", error),
    };*/

    wallet

    /*match sync_result {
        Ok(_) => return wallet,
        Err(error) => println!("Problem synchronizing wallet: {:?}", error),
    };*/

    // Fetch a fresh address to receive coins
    // let address = wallet.get_address(AddressIndex::New).unwrap().address;

    // println!("address: {:?}", address);

    // false
}

fn get_descriptors(network: &Network, mnemonic_words: &String) -> (String, String) {

    // Parse a mnemonic
    let mnemonic  = Mnemonic::parse(mnemonic_words).unwrap();

    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();

    // Get xprv from the extended key
    let xprv = xkey.into_xprv(*network).unwrap();

    // Create derived privkey from the above xpriv (master key)
    // The following dereivation paths will be used for receive and change keys
    // receive: "m/84h/1h/0h/0"
    // change: "m/84h/1h/0h/1"
    let mut keys= Vec::new();

    // Create a new Secp256k1 context
    let secp = Secp256k1::new();

    for path in ["m/84h/1h/0h/0", "m/84h/1h/0h/1"] {
        let derived_path = DerivationPath::from_str(path).unwrap();
        let derived_xpriv = &xprv.derive_priv(&secp, &derived_path).unwrap();
        let origin: KeySource = (xprv.fingerprint(&secp), derived_path);
        let derived_xprv_descriptor_key: DescriptorKey<Segwitv0> =
            derived_xpriv.into_descriptor_key(Some(origin), DerivationPath::default()).unwrap();


        // Wrap the derived key with the "wpkh()" string to produce a descriptor text
        if let Secret(key, _, _) = derived_xprv_descriptor_key {
            let mut desc = "wpkh(".to_string();
            desc.push_str(&key.to_string());
            desc.push_str(")");
            keys.push(desc);
        }
    }

    (keys[0].clone(), keys[1].clone())
}

fn main() {

    let network = Network::Signet;

    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();

    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();

    let (receive_desc, change_desc) = get_descriptors(&network, &mnemonic_words);
    println!("mnemonic: {}\n\nrecv desc: {:#?}\n\nchng desc: {:#?}", mnemonic_words, receive_desc, change_desc);


    if !test_bitcoin_rpc_connection() {
        return;
    }

    /*let block_count = get_block_count();

    println!("block count: {}", block_count);*/

    let block_count = u32::try_from(get_block_count()).ok().unwrap();


    let wallet = create_wallet(&network, &receive_desc, &change_desc, block_count);

    // Fetch a fresh address to receive coins
    let address = wallet.get_address(AddressIndex::New).unwrap().address;

    println!("address: {}", address);

    //test_bitcoin_rpc_connection(wallet_name: String)

    wallet_management::load_wallet("3p92ujcyqvky050s".to_string());
}
