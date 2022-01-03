#![allow(unused_imports)]

use bdk::bitcoin::Network;

mod wallet_electrum;
mod wallet_core;
mod wallet_common;
mod wallet_neutrino;

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

    // Choose connection method

    // run_rpc_core();

    // run_neutrino();

    run_electrum();
}
