// Copyright 2015 MaidSafe.net limited
//
// This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the MaidSafe Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
// directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
// available at: http://www.maidsafe.net/licenses
//
// Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
// OF ANY KIND, either express or implied.
//
// See the Licences for the specific language governing permissions and limitations relating to
// use of the MaidSafe Software.

//! The main API for routing nodes (this is where you give the network it's rules)
//! The network will report **From Authority your Authority** and validate cryptographically
//! and via group consensus any message. This means any facade you implement will set out
//! what you deem to be a valid operation, routing will provide a valid message sender and authority
//! that will allow you to set up many decentralised services
//! See maidsafe.net to see what they are doing as an example
//!
//! The data types are encoded with Concise Binary Object Representation (CBOR)
//! This allows us to demand certain tags are available to routing that allows
//! it to confirm things like data.name() when calculating authority
//! We use Iana tag representations http://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml
//! Please define your own for this library. These tags are non optional and your data MUST meet
//! the requirements and implement the following tags
//! tag: 5483_0 -> name [u8; 64] type
//! tag: 5483_1 -> XXXXXXXXXXXXXX
//! # Use

#![feature(collections)]
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/routing")]
// #![warn(missing_docs)]
#![allow(dead_code, unused_variables, unused_features)]
#![feature(custom_derive, rand, collection, std_misc, unsafe_destructor, unboxed_closures, io, core,
           udp, thread_sleep, ip_addr)]

extern crate sodiumoxide;
extern crate lru_cache;
extern crate rustc_serialize;
extern crate cbor;
extern crate rand;
extern crate time;
extern crate bchannel;
extern crate sqlite3;

use std::net::{TcpStream};
use sodiumoxide::crypto;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::default::Default;

pub mod types;
pub mod tcp_connections;
mod connection_manager;
mod broadcast;
mod message_header;
pub mod routing_table;
mod accumulator;
mod common_bits;
mod sentinel;
mod bootstrap;
mod messages;

//#[derive(RustcEncodable, RustcDecodable)]
struct SignedKey {
  sign_public_key: crypto::sign::PublicKey,
  encrypt_public_key: crypto::asymmetricbox::PublicKey,
  signature: crypto::sign::Signature // detached signature
}

pub struct DestinationAddress {
  pub dest: Vec<u8>
}

//#[derive(RustcEncodable, RustcDecodable, Default)]
pub struct DhtIdentity {
  pub id: [u8; 64]
}

impl Default for DhtIdentity {
  #[inline]
  fn default()->DhtIdentity {
    DhtIdentity { id: [0; 64] }
  }
}

impl DhtIdentity {
  /* fn name(&self) { */
  /*  msgpack::Encoder::to_msgpack(&self.signed_key).ok().unwrap()  */
  /* }   */

}

pub enum Authority {
  Client,
  Node,
  ClientManager,
  NaeManager,
  NodeManager
}

pub enum Action {
  Reply(Vec<u8>),
  SendOn(Vec<DhtIdentity>)
}


pub enum RoutingError {
  Success,  // vault will aslo return a Success to indicate a deadend
  NoData,
  InvalidRequest,
  IncorrectData(Vec<u8>)
}

pub trait Facade : Sync {
  /// if reply is data then we send back the response message (ie get_response )
  fn handle_get(&mut self, our_authority: Authority, from_authority: Authority, from_address: DhtIdentity, data: Vec<u8>)->Result<Action, RoutingError>;

  // TODO : datatype needs to be passed, or the type of data shall be Data (name + content) instead of serialised_data
  fn handle_put(&mut self, our_authority: Authority, from_authority: Authority,
                from_address: DhtIdentity, dest_address: DestinationAddress, data: Vec<u8>)->Result<Action, RoutingError>;

  fn handle_post(&mut self, our_authority: Authority, from_authority: Authority, from_address: DhtIdentity, data: Vec<u8>)->Result<Action, RoutingError>;
  fn handle_get_response(&mut self, from_address: DhtIdentity, response: Result<Vec<u8>, RoutingError>);
  fn handle_put_response(&mut self, from_authority: Authority, from_address: DhtIdentity, response: Result<Vec<u8>, RoutingError>);
  fn handle_post_response(&mut self, from_authority: Authority, from_address: DhtIdentity, response: Result<Vec<u8>, RoutingError>);
  }

/// DHT node
pub struct RoutingNode<'a> {
facade: &'a (Facade + 'a),
sign_public_key: crypto::sign::PublicKey,
sign_secret_key: crypto::sign::SecretKey,
encrypt_public_key: crypto::asymmetricbox::PublicKey,
encrypt_secret_key: crypto::asymmetricbox::SecretKey,
sender: Sender<TcpStream>,
receiver: Receiver<TcpStream>
}

impl<'a> RoutingNode<'a> {
  pub fn new(my_facade: &'a Facade) -> RoutingNode<'a> {
    sodiumoxide::init(); // enable shared global (i.e. safe to mutlithread now)
    let key_pair = crypto::sign::gen_keypair();
    let encrypt_key_pair = crypto::asymmetricbox::gen_keypair();
    let (tx, rx) : (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel();

    RoutingNode { facade: my_facade,
                  sign_public_key: key_pair.0, sign_secret_key: key_pair.1,
                  encrypt_public_key: encrypt_key_pair.0, encrypt_secret_key: encrypt_key_pair.1, sender: tx, receiver: rx }
  }

  /// Retreive something from the network (non mutating) - Direct call
  pub fn get(&self, name: types::DhtAddress) { unimplemented!()}

  /// Add something to the network, will always go via ClientManager group
  pub fn put(&self, name: types::DhtAddress, content: Vec<u8>) { unimplemented!() }

  /// Mutate something on the network (you must prove ownership) - Direct call
  pub fn post(&self, name: types::DhtAddress, content: Vec<u8>) { unimplemented!() }

  pub fn start() {

  }

  fn add_bootstrap(&self) {}


  fn get_facade(&'a mut self) -> &'a Facade {
    self.facade
  }
}


#[test]
fn facade_implementation() {

  struct MyFacade;

  impl Facade for MyFacade {
    fn handle_get(&mut self, our_authority: Authority, from_authority: Authority,from_address: DhtIdentity , data: Vec<u8>)->Result<Action, RoutingError> { unimplemented!(); }
    fn handle_put(&mut self, our_authority: Authority, from_authority: Authority,
                  from_address: DhtIdentity, dest_address: DestinationAddress, data: Vec<u8>)->Result<Action, RoutingError> { unimplemented!(); }
    fn handle_post(&mut self, our_authority: Authority, from_authority: Authority, from_address: DhtIdentity, data: Vec<u8>)->Result<Action, RoutingError> { unimplemented!(); }
    fn handle_get_response(&mut self, from_address: DhtIdentity , response: Result<Vec<u8>, RoutingError>) { unimplemented!() }
    fn handle_put_response(&mut self, from_authority: Authority,from_address: DhtIdentity , response: Result<Vec<u8>, RoutingError>) { unimplemented!(); }
    fn handle_post_response(&mut self, from_authority: Authority,from_address: DhtIdentity , response: Result<Vec<u8>, RoutingError>) { unimplemented!(); }
  }
  let my_facade = MyFacade;
  let my_routing = RoutingNode::new(&my_facade);
  /* assert_eq!(999, my_routing.get_facade().handle_get_response());  */
}
