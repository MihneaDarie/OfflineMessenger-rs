
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IP {
    V4(u8, u8, u8, u8),
    V6(u8, u8, u8, u8, u8, u8),
}

impl IP {
    pub fn get_ipv4(&self) -> Option<(u8, u8, u8, u8)> {
        match self {
            IP::V4(n1, n2, n3, n4) => Some((*n1, *n2, *n3, *n4)),
            _ => None,
        }
    }
    pub fn get_ipv6(&self) -> Option<(u8, u8, u8, u8, u8, u8)> {
        match self {
            IP::V6(n1, n2, n3, n4, n5, n6) => Some((*n1, *n2, *n3, *n4, *n5, *n6)),
            _ => None,
        }
    }
}

pub struct ServerDetails {
    ip_adress: IP,
    port: u16,
}

impl ServerDetails {
    pub fn new_ipv4(n1: u8, n2: u8, n3: u8, n4: u8, port: u16) -> Self {
        Self {
            ip_adress: IP::V4(n1, n2, n3, n4),
            port: port,
        }
    }
    pub fn new_ipv6(n1: u8, n2: u8, n3: u8, n4: u8,n5: u8, n6: u8, port: u16) -> Self {
        Self {
            ip_adress: IP::V6(n1, n2, n3, n4, n5, n6),
            port: port,
        }
    }
    pub fn ip_to_string(&self) -> Option<String> {
        match self.ip_adress.get_ipv4() {
            Some((a, b, c, d)) => {
                let s = format!("{}.{}.{}.{}", a, b, c, d);
                return Some(s);
            }
            None => match self.ip_adress.get_ipv6() {
                Some((a, b, c, d, e, f)) => {
                    let s = format!("{}.{}.{}.{}.{}.{}", a, b, c, d, e, f);
                    return Some(s);
                }
                None => None,
            },
        }
    }

    pub fn port_to_string(&self) -> Option<String> {
        let s = format!("{}",self.port);
        Some(s)
    }
}
