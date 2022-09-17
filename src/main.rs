use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use clap::{App, Arg};
use rand;
use trust_dns::op::{Message, OpCode, Query,MessageType};
use trust_dns::proto::rr::record_type::RecordType;
use trust_dns::rr::domain::Name;
use trust_dns::serialize::binary::*;

///一个命令行工具 可以解析域名对应的ip地址
fn main() {
    let app = App::new("resolve")
        .about("一个命令行工具")
        .arg(Arg::with_name("dns-server").short("s").default_value("1.1.1.1"))
        .arg(Arg::with_name("domain-name").required(true))
        .get_matches();

    //把命令行参数转化为一个有类型的域名
    let domain_name = app.value_of("domain-name").unwrap();
    let domain_name = Name::from_ascii(&domain_name).unwrap();

    //把命令行参数转化为一个有类型的DNS服务器
    let dns_server = app.value_of("dns-server").unwrap();
    let dns_server:SocketAddr = format!("{}:53",dns_server).parse().expect("invalid address");

    let mut request_as_bytes = Vec::with_capacity(512); //长度0 容量512
    let mut response_as_bytes = vec![0;512]; //长度512

    //定义一个Message DNS报文 可以保存查询 也可以保存应答
    let mut msg = Message::new();
    msg.set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name,RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    //把Message转化为原始字节
    let mut encode = BinEncoder::new(&mut request_as_bytes);
    msg.emit(&mut encode).unwrap();

    let localhost = UdpSocket::bind("0.0.0.0:0").expect("cannot bind to local socket");
    let timeout = Duration::from_secs(3);
    localhost.set_read_timeout(Some(timeout)).unwrap();
    localhost.set_nonblocking(false).unwrap();

    localhost.send_to(&request_as_bytes,dns_server).unwrap();

    let (_amt,_remote) = localhost.recv_from(&mut response_as_bytes).expect("time out");

    let dns_message = Message::from_vec(&response_as_bytes).expect("unable parse");
    for answer in dns_message.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.rdata();
            let ip = resource.to_ip_addr().expect("invalid IP address received");
            println!("{}",ip.to_string())
        }
    }
}
