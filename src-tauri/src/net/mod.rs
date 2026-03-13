use std::net::UdpSocket;

pub fn test_client() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;



    Ok(())
}
