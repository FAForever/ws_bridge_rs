use clap::{Arg, Command};

use tokio;

use env_logger::{Builder, WriteStyle};
use log::LevelFilter;

type Error = Box<dyn std::error::Error>;

mod common;

// helpful example; https://github.com/snapview/tokio-tungstenite/issues/137

fn main() -> Result<(), Error> {
    let cmd =
        Command::new("Websocket Bridge")
            .about("Allows bridging a TCP connection over a websocket.")
            .arg(
                Arg::new("v")
                    .short('v')
                    .action(clap::ArgAction::Count)
                    .value_parser(clap::value_parser!(u8).range(0..5))
                    .help("Increases the verbosity"),
            )
            .arg(
                Arg::new("proxy")
                    .long("proxy")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enables PROXY protocol when running in ws_to_tcp mode"),
            )
            .arg(
                Arg::new("proxy-header-name")
                    .long("proxy-header-name")
                    .default_value("X-Forwarded-For")
                    .requires("proxy")
                    .help("Get the original IP from this header when running in PROXY mode"),
            )
            .arg(
                Arg::new("mode")
                    .value_parser(["ws_to_tcp", "tcp_to_ws"])
                    .required(true)
                    .help("The direction of transfer."),
            )
            .arg(Arg::new("bind").required(true).help("ip:port to bind to."))
            .arg(Arg::new("dest").required(true).help(
                "ip:port to send to (for websockets; ip:port, [ws[s]://]example.com/sub/path)",
            ));

    let matches = cmd.clone().get_matches();

    let verbosity = matches.get_count("v");
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => panic!("`v` must be in range 0..5"),
    };

    let _stylish_logger = Builder::new()
        .filter(None, level)
        .write_style(WriteStyle::Always)
        .init();

    let bind_value = matches
        .get_one::<String>("bind")
        .expect("`bind` is required");

    let dest_value = matches
        .get_one::<String>("dest")
        .expect("`dest` is required");

    let rt = tokio::runtime::Runtime::new().unwrap();

    let proxy = matches.get_flag("proxy");
    let proxy_header_name = matches.get_one::<String>("proxy-header-name").unwrap();
    let direction = match matches
        .get_one::<String>("mode")
        .expect("`mode` is required")
        .as_str()
    {
        "ws_to_tcp" => common::Direction::WsToTcp,
        "tcp_to_ws" => common::Direction::TcpToWs,
        &_ => {
            panic!("Got unknown direction, shouldn't be possible.");
        }
    };

    rt.block_on(async {
        let res = common::serve(bind_value, dest_value, direction, proxy, proxy_header_name).await;
        panic!("Serve returned with {:?}", res);
    });

    Ok(())
}
