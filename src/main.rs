use structopt::StructOpt;
use std::net::{IpAddr};

#[derive(StructOpt, Debug)]
#[structopt(name = "ping-util-rs", author = "Dishon Merkhai", no_version, about = "This program is a Rust implementation of the UNIX ping command")]
pub struct Opt {
  // Activate debug mode
  #[structopt(short, long, help ="Activate debug mode")]
  debug: bool,

  // The IP address to ping
  // ! need to implement this without requiring --ip
  #[structopt(required = true, long, help = "The IP address (IPv4, IPv6) to send packets towards")]
  ip: IpAddr,

  // Specify TTL (Time-To-Live), reports ICMP messages that have exceeded the set TTL
  // ! need to implement this without requiring --ttl
  #[structopt(short, long, default_value = "255", help = "Set Time to live (TTL) and report packets that have exceeded the TTL")]
  ttl: u8,

  // Terminate after sending (and receiving) count ECHO_RESPONSE packets.
  #[structopt(short = "c", long = "count", default_value = "-1", help = "Terminates after sending (and receiving) count ECHO_RESPONSE packets")]
  max_packets: i32,

  // 
  #[structopt(short = "s", default_value = "56", help = "Specify the number of data bytes to be sent.  The default is 56,
             which translates into 64 ICMP data bytes when combined with the 8
             bytes of ICMP header data.  This option cannot be used with ping
             sweeps.")]
  packet_size: i32, // TODO: should be u32, but the library implementation has i32

  // 
  #[structopt(short = "i", long = "rtt", default_value = "1000", help = "Wait wait_time milliseconds between sending each packet.  The default is to
             wait for one second between each packet.")]
  max_rtt: u64,
}

pub fn summary(_opt: &Opt, _icmp_seq: u32, _failed_packets: u32) {
  println!("\n--- {} ping statistics ---", _opt.ip);
  println!("{} packets transmitted, {} packets received, {:.3?}% packet loss", _icmp_seq, (_icmp_seq - _failed_packets), ((_failed_packets/_icmp_seq)*100));
  println!("round-trip min/avg/max/stddev = 0.000/0.000/0.000/0.000 ms") // {:.3?}
}

fn main() {
  let opt = Opt::from_args();
  // println!("{:#?}", opt);

  // experimental implementation provided by fastping_rs documentation
  env_logger::init();
  let (pinger, results) = match ping_util_rs::Pinger::new(Some(opt.max_rtt), Some(opt.packet_size), Some(opt.ttl), opt.ip.is_ipv4()) {
      Ok((pinger, results)) => (pinger, results),
      Err(e) => panic!("Error creating pinger: {}", e)
  };


  pinger.add_ipaddr(&opt.ip.to_string());
  pinger.run_pinger();
  // add 8 for the ICMP header size (8 bytes)
  let send_size: i32 = pinger.get_size() + 8;

  let mut icmp_seq: u32 = 0;
  let mut failed_packets: u32 = 0;

  println!("PING {} ({}): {} data bytes", opt.ip, opt.ip, pinger.get_size());

  let mut x: i32 = 0;

  while opt.max_packets != x {
    match results.recv() {
        Ok(result) => {
            icmp_seq += 1;
            match result {
                ping_util_rs::PingResult::Idle{addr} => {
                    log::error!("TTL Time Exceeded from {}: icmp_seq={} payload={}B", addr, icmp_seq, send_size);
                    failed_packets += 1;
                },
                ping_util_rs::PingResult::Receive{addr, rtt} => {
                    println!("{} bytes from {}: icmp_seq={} ttl={} rtt={:.5?} loss={}%", send_size, addr, icmp_seq, opt.ttl, rtt, ((failed_packets/icmp_seq)*100));
                }
            }
        },
        Err(_) => panic!("Worker threads disconnected before the solution was found!"),
    }
    x += 1;
  }
  // stop the pinger device
  pinger.stop_pinger();

  summary(&opt, icmp_seq, failed_packets);

}