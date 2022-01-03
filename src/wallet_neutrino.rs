use std::sync::Arc;

use bdk::{blockchain::{compact_filters::{Mempool, Peer, CompactFiltersError}, CompactFiltersBlockchain, noop_progress}, bitcoin::Network, Wallet, database::MemoryDatabase, wallet::AddressIndex};

use crate::wallet_common::{get_descriptors, build_signed_tx};

fn load_or_create_wallet(neutrino_url: &str, network: &Network, receive_desc: &String, change_desc: &String) -> Wallet<CompactFiltersBlockchain, MemoryDatabase> {

    let mempool = Arc::new(Mempool::default());

    /*let peer = Peer::connect(
            neutrino_url,
            Arc::clone(&mempool),
            *network,
        ).unwrap();

    let peers = vec![peer];*/

    let num_threads = 4;

    let peers = (0..num_threads)
        .map(|_| Peer::connect(
            neutrino_url,
            Arc::clone(&mempool),
            Network::Testnet)
        )
        .collect::<Result<_, _>>().unwrap();

    let blockchain = CompactFiltersBlockchain::new(
        peers,
        "./wallet-filters",
        Some(2_100_100) // Some(700_000)
    ).unwrap();

    // println!("done {:?}", blockchain);

    let wallet = Wallet::new(
        receive_desc,
        Some(change_desc),
        *network,
        MemoryDatabase::default(),
        blockchain
    ).unwrap();

    wallet.sync(noop_progress(), None).unwrap();

    wallet
}

pub fn run(network: Network, neutrino_url: &str, mnemonic_words: &str) {

    let (receive_desc, change_desc) = get_descriptors(&network, mnemonic_words);

    let wallet = load_or_create_wallet(neutrino_url, &network, &receive_desc, &change_desc);

    let address = wallet.get_address(AddressIndex::New).unwrap().address;

    println!("address: {}", address);

    let balance = wallet.get_balance().unwrap();

    println!("balance: {}", balance);

    if balance > 100 {

        let recipient_address = "<addr>";

        let amount = 9359;

        let tx = build_signed_tx(&wallet, recipient_address, amount);

        let tx_id = wallet.broadcast(&tx).unwrap();

        println!("tx id: {}", tx_id.to_string());
    } else {
        println!("Insufficient Funds. Fund the wallet with the address above");
    }

    println!("------\nmnemonic: {}\n\nrecv desc: {:#?}\n\nchng desc: {:#?}", mnemonic_words, receive_desc, change_desc);


}