use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{DerivationPath, KeySource, ExtendedPrivKey};
use bdk::bitcoin::{Network, Address, Transaction};
use bdk::bitcoincore_rpc::{Client, Auth as core_rpc_auth, RpcApi};
use bdk::blockchain::{ConfigurableBlockchain, NoopProgress, LogProgress};
use bdk::database::BatchDatabase;
use bdk::descriptor::Segwitv0;
use bdk::keys::{GeneratableKey, DescriptorKey, ExtendedKey};
use bdk::template::Bip84;
use bdk::{keys::GeneratedKey};
use bdk::keys::bip39::{Mnemonic, WordCount, Language};
use bdk::{miniscript, KeychainKind};
use bdk::keys::{DerivableKey, DescriptorKey::Secret};

use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig};

use bdk::Wallet;
use bdk::wallet::{AddressIndex, signer::SignOptions, wallet_name_from_descriptor};

use bdk::sled::{self, Tree};

use std::str::FromStr;

pub fn mnemonic_to_xprv(network: &Network, mnemonic_words: &str) -> ExtendedPrivKey {
    // Parse a mnemonic
    let mnemonic  = Mnemonic::parse(mnemonic_words).unwrap();

    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();

    // Get xprv from the extended key
    let xprv = xkey.into_xprv(*network).unwrap();

    xprv
}

pub fn get_descriptors(network: &Network, mnemonic_words: &str) -> (String, String) {

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

pub fn build_signed_tx<B, D: BatchDatabase>(wallet: &Wallet<B, D>, recipient_address: &str, amount: u64) -> Transaction {
    // Create a transaction builder
    let mut tx_builder = wallet.build_tx();

    let to_address = Address::from_str(recipient_address).unwrap();

    // Set recipient of the transaction
    tx_builder.set_recipients(vec!((to_address.script_pubkey(), amount)));

    // Finalise the transaction and extract PSBT
    let (mut psbt, _) = tx_builder.finish().unwrap();

    // Sign the above psbt with signing option
    wallet.sign(&mut psbt, SignOptions::default()).unwrap();

    // Extract the final transaction
    let tx = psbt.extract_tx();

    tx
}