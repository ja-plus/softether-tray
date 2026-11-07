use softether_tray_lib::softether::AccountSummary;
use softether_tray_lib::vpn;

#[test]
fn live_detect_iface() {
    // 用本机真实账户清单：账户 tet 连着，适配器 VPN
    let accounts = vec![AccountSummary {
        name: "tet".into(),
        status: "已连接".into(),
        host: "hzvpn.tracyenergy.cn".into(),
        port: Some(12443),
        hub: "tracyhz".into(),
        nic: "VPN".into(),
    }];
    let ipv4 = softether_tray_lib::settings::ipv4_addresses();
    println!("detected = {:?}", vpn::detect_iface(&accounts, &[], &ipv4));
    assert_eq!(vpn::detect_iface(&accounts, &[], &ipv4).as_deref(), Some("vpn_vpn"));
}
