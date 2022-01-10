#![allow(unused_imports)]

use bdk::{bitcoin::Network, keys::{GeneratedKey, bip39::{Mnemonic, WordCount, Language}}, miniscript};
use bdk::keys::GeneratableKey;

mod wallet_electrum;
mod wallet_core;
mod wallet_common;
mod wallet_neutrino;
mod wallet_esplora;

fn run_rpc_core() {
    let network = Network::Signet;

    let mnemonic_words = "stuff will crystal safe camera engage cereal measure gaze abstract mass embrace";

    let rpc_url = "127.0.0.1:38332";

    let username = "admin";
    let password = "password";

    wallet_core::run(network, rpc_url, username, password, mnemonic_words);
}

fn run_electrum() {
    let network = Network::Testnet;

    let mnemonic_words = "stuff will crystal safe camera engage cereal measure gaze abstract mass embrace";

    let electrum_url = "ssl://electrum.blockstream.info:60002";

    wallet_electrum::run(network, electrum_url, mnemonic_words);
}

fn run_esplora() {
    let network = Network::Testnet;

    let mnemonic_words = "stuff will crystal safe camera engage cereal measure gaze abstract mass embrace";

    let esplora_url = "https://blockstream.info/testnet/api";

    wallet_esplora::run(network, esplora_url, mnemonic_words);
}

fn run_neutrino() {
    // let network = Network::Bitcoin;
    let network = Network::Testnet;

    // let neutrino_url = "btcd-mainnet.lightning.computer:8333";
    // let neutrino_url = "bb1.breez.technology:8333";
    let neutrino_url = "faucet.lightning.community:18333";

    let mnemonic_words = "stuff will crystal safe camera engage cereal measure gaze abstract mass embrace";

    wallet_neutrino::run(network, neutrino_url, mnemonic_words)
}

fn main() {

    env_logger::init();

    // Choose connection method

    // run_rpc_core();

    // run_neutrino();

    // run_electrum();

    run_esplora();

    /*
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();

    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();

    let (receive_desc, change_desc) = wallet_common::get_descriptors(&Network::Testnet, &mnemonic_words);
    println!("mnemonic: {}\n\nrecv desc: {:#?}\n\nchng desc: {:#?}", mnemonic_words, receive_desc, change_desc);
    */
}
