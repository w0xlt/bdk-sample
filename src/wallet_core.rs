use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{DerivationPath, KeySource, ExtendedPrivKey};
use bdk::bitcoin::{Network, Address, Transaction};
use bdk::bitcoincore_rpc::{Client, Auth as core_rpc_auth, RpcApi};
use bdk::blockchain::{ConfigurableBlockchain, NoopProgress, LogProgress};
use bdk::descriptor::Segwitv0;
use bdk::keys::{GeneratableKey, DescriptorKey, ExtendedKey};
use bdk::template::Bip84;
use bdk::wallet::export::WalletExport;
use bdk::{keys::GeneratedKey};
use bdk::keys::bip39::{Mnemonic, WordCount, Language};
use bdk::{miniscript, KeychainKind};
use bdk::keys::{DerivableKey, DescriptorKey::Secret};

use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};

use bdk::Wallet;
use bdk::wallet::{AddressIndex, signer::SignOptions, wallet_name_from_descriptor};

use bdk::sled::{self, Tree};

use std::str::FromStr;

use crate::wallet_common::{get_descriptors, build_signed_tx, mnemonic_to_xprv};

fn load_or_create_wallet(network: &Network, rpc_url: &str, username: &str, password: &str, xpriv: &ExtendedPrivKey) -> Wallet<RpcBlockchain, Tree> {

    let auth = Auth::UserPass {
        username: username.to_string(),
        password: password.to_string()
    };

    // Use deterministic wallet name derived from descriptor
    let wallet_name = wallet_name_from_descriptor(
        Bip84(xpriv.clone(), KeychainKind::External),
        Some(Bip84(*xpriv, KeychainKind::Internal)),
        *network,
        &Secp256k1::new()
    ).unwrap();

    println!("wallet name: {:?}", wallet_name);

    // Setup the RPC configuration
    let rpc_config = RpcConfig {
        url: rpc_url.to_string(),
        auth,
        network: *network,
        wallet_name: wallet_name.clone(),
        skip_blocks: Some(70_000) // Some(block_count)
    };

    // Use the above configuration to create a RPC blockchain backend
    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();

    // Create the datadir to store wallet data
    let mut datadir = dirs_next::home_dir().unwrap();
    datadir.push(".bdk-wallets");
    datadir.push(wallet_name.clone());
    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(wallet_name.clone()).unwrap();

    // Combine everything and finally create the BDK wallet structure
    let wallet = Wallet::new(
        Bip84(xpriv.clone(), KeychainKind::External),
        Some(Bip84(*xpriv, KeychainKind::Internal)),
        *network,
        db_tree,
        blockchain
    ).unwrap();

    // Sync the wallet
    //let sync_result = wallet.sync(LogProgress, None);
    wallet.sync(LogProgress, None).unwrap();

    wallet
}

pub fn run(network: Network, rpc_url: &str, username: &str, password: &str, mnemonic_words: &str) {

    let xpriv = mnemonic_to_xprv(&network, &mnemonic_words);

    let wallet = load_or_create_wallet(&network, rpc_url, username, password, &xpriv);

    println!("mnemonic: {}\n\nrecv desc (pub key): {:#?}\n\nchng desc (pub key): {:#?}",
    mnemonic_words,
    wallet.get_descriptor_for_keychain(KeychainKind::External).to_string(),
    wallet.get_descriptor_for_keychain(KeychainKind::Internal).to_string());

    // Fetch a fresh address to receive coins
    let address = wallet.get_address(AddressIndex::New).unwrap().address;

    println!("new address: {}", address);

    let balance = wallet.get_balance().unwrap();

    println!("balance: {}", balance);

    if balance > 100 {

        let recipient_address = "<addr>";

        let tx = build_signed_tx(&wallet, recipient_address, 5000);

        // Broadcast the transaction
        let tx_id = wallet.broadcast(&tx).unwrap();


        println!("tx id: {}", tx_id.to_string());

    } else {
        println!("Insufficient Funds. Fund the wallet with the address above");
    }

    let export = WalletExport::export_wallet(&wallet, "exported wallet", true)
        .map_err(ToString::to_string)
        .map_err(bdk::Error::Generic).unwrap();

    println!("------\nWallet Backup: {}", export.to_string());
}