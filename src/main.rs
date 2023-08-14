use log::{debug, error, info, LevelFilter};
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, str};
use tokio::time::sleep;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

use crate::model::{TunnelParams, TunnelType, UserConfig};

mod model;

fn main() {
    dioxus_logger::DioxusLogger::new(LevelFilter::Info)
        .use_format("[{LEVEL}] {PATH} - {ARGS}")
        .build()
        .expect("Failed to initialize logger");
    const MAX_PACKET_SIZE: usize = 1_000_000;
    const USER_CONF_PATH: &str = "user-config.json";
    static SERVER_ADDRESS: &str = "127.0.0.1:7779";
    let title = "snx-rs-gui";
    let user_config = read_config().unwrap_or_else(|| UserConfig {
        tunnel_params: TunnelParams::default(),
        remember_me: false,
    });
    info!("Starting application");
    dioxus_desktop::launch_with_props(
        app,
        user_config,
        Config::default().with_window(
            WindowBuilder::new()
                .with_title(title)
                .with_resizable(true)
                .with_inner_size(dioxus_desktop::wry::application::dpi::LogicalSize::new(
                    600.0, 600.0,
                )),
        ),
    );

    fn app(cx: Scope<UserConfig>) -> Element {
        let config = cx.props.tunnel_params.clone();

        let username = use_ref(cx, || config.user_name);
        let password = use_ref(cx, String::new);
        let server_address = use_ref(cx, || config.server_name);
        let log_level = use_ref(cx, || config.log_level);
        let reauth = use_state(cx, || config.reauth);
        let search_domains = use_ref(cx, || config.search_domains);
        let default_route = use_state(cx, || config.default_route);
        let no_routing = use_state(cx, || config.no_routing);
        let no_dns = use_state(cx, || config.no_dns);
        let no_cert_check = use_state(cx, || config.no_cert_check);
        let tunnel_type = use_ref(cx, || config.tunnel_type);
        let ca_cert = use_ref(cx, || config.ca_cert);
        let login_type = use_ref(cx, || config.login_type);

        let remember_me = use_state(cx, || cx.props.remember_me);

        let settings_expanded = use_state(cx, || false);

        let missing_username = use_state(cx, || false);
        let missing_password = use_state(cx, || false);
        let missing_server_address = use_state(cx, || false);

        let status = use_state(cx, || false);
        let connection_status = use_state(cx, || false);

        let status_msg = "Snx-rs service status: ";
        let connection_status_msg = "Connection status: ";

        status_service(cx, status, connection_status);

        let current_settings = || TunnelParams {
            user_name: username.read().to_string(),
            server_name: server_address.read().to_string(),
            password: password.read().to_string(),
            log_level: log_level.read().to_string(),
            reauth: reauth.get().to_owned(),
            search_domains: search_domains.read().to_owned(),
            default_route: default_route.get().to_owned(),
            no_routing: no_routing.get().to_owned(),
            no_dns: no_dns.get().to_owned(),
            no_cert_check: no_cert_check.get().to_owned(),
            tunnel_type: tunnel_type.read().to_owned(),
            ca_cert: ca_cert.read().to_owned(),
            login_type: login_type.read().to_owned(),
        };

        if remember_me.get().to_owned() {
            let save_res = save_config(UserConfig {
                tunnel_params: current_settings(),
                remember_me: remember_me.get().to_owned(),
            });
            info!("Saving config result: {:?}", save_res);
        };

        cx.render(rsx! {
            div {
                link {
                    href: "https://fonts.googleapis.com/icon?family=Material+Icons",
                    rel: "stylesheet"
                }
                style { include_str!("./style.css") }
            }
            main {
                div {
                    class: "login-container",
                    onclick: move |_| {
                        settings_expanded.set(false);
                    },
                    div { class: "login-form",
                        input {
                            placeholder: "Username",
                            class: "form-input",
                            value: "{username.read()}",
                            oninput: move |e| {
                                username.set(e.value.clone());
                            }
                        }
                        input {
                            placeholder: "Password",
                            class: "form-input",
                            r#type: "password",
                            value: "{password.read()}",
                            oninput: move |e| {
                                password.set(e.value.clone());
                            }
                        }
                        div { class: "button-container",
                            // TODO trigger on enter
                            button {
                                // TODO trigger on enter
                                class: "form-button connect",
                                disabled: if **connection_status { true } else { false },
                                // TODO trigger on enter
                                onclick: move |_| {
                                    let username_string = username.read().to_string();
                                    let password_string = password.read().to_string();
                                    let server_address_string = server_address.read().to_string();
                                    let mut error_state = false;
                                    if username_string.is_empty() {
                                        missing_username.set(true);
                                        error_state = true;
                                    } else {
                                        missing_username.set(false);
                                    }
                                    if password_string.is_empty() {
                                        missing_password.set(true);
                                        error_state = true;
                                    } else {
                                        missing_password.set(false);
                                    }
                                    if server_address_string.is_empty() {
                                        missing_server_address.set(true);
                                        error_state = true;
                                    } else {
                                        missing_server_address.set(false);
                                    }
                                    if error_state {
                                        return;
                                    }
                                    settings_expanded.set(false);
                                    connect(current_settings());
                                    password.set("".to_string());
                                },
                                "Connect"
                            }
                            button {
                                class: "form-button disconnect",
                                disabled: if **connection_status { false } else { true },
                                onclick: move |_| {
                                    disconnect();
                                },
                                "Disconnect"
                            }
                        }
                    }
                    div { class: "remember-me",
                        input {
                            r#type: "checkbox",
                            checked: if **remember_me { "true" } else { "false" },
                            oninput: move |e| {
                                let checked = match e.value.as_str() {
                                    "true" => true,
                                    "false" => false,
                                    _ => false,
                                };
                                save_config(UserConfig {
                                        tunnel_params: current_settings(),
                                        remember_me: checked,
                                    })
                                    .unwrap_or_else(|e| {
                                        error!("Error: {}", e.to_string());
                                    });
                                remember_me.set(checked);
                            }
                        }
                        span { "Remember configuration" }
                    }
                    div { class: "error-container",
                        span { class: "error-text", display: if **missing_username { "block" } else { "none" }, "Error: Username is required" }
                        span { class: "error-text", display: if **missing_password { "block" } else { "none" }, "Error: Password is required" }
                        span { class: "error-text", display: if **missing_server_address { "block" } else { "none" }, "Error: Server adress is required" }
                    }
                    div { class: "status",
                        span { class: "status-text", connection_status_msg.to_string() }
                        span { class: if **connection_status { "status-text-green" } else { "status-text-red" },
                            if **connection_status{ "connected" } else { "disconected" }
                        }
                        br {}
                        span { class: "status-text", status_msg.to_string() }
                        span { class: if **status { "status-text-green" } else { "status-text-red" },
                            if **status { "running" } else { "stopped" }
                        }
                    }
                }
                img {
                    src: "../lib/snx-rs-gui/assets/settings_white.png",
                    class: "settings-icon",
                    onclick: move |_| {
                        settings_expanded.set(!settings_expanded.get());
                    }
                }
                div { class: "settings-panel", display: if **settings_expanded { "block" } else { "none" },
                    h3 { "Settings" }
                    ul {
                        li {
                            "Server adress"
                            input {
                                placeholder: "",
                                class: "settings-form-input",
                                value: "{server_address.read()}",
                                oninput: move |e| { server_address.set(e.value.clone()) }
                            }
                        }
                        li {
                            "Log level"
                            select {
                                value: "{log_level.read()}",
                                onchange: move |selection| {
                                    log_level.set(selection.data.value.clone());
                                },
                                option { "debug" }
                                option { "info" }
                                option { "warn" }
                                option { "error" }
                            }
                        }
                        li {
                            "Reauthorization"
                            input {
                                r#type: "checkbox",
                                checked: if **reauth { "true" } else { "false" },
                                oninput: move |e| {
                                    match e.value.as_str() {
                                        "true" => reauth.set(true),
                                        "false" => reauth.set(false),
                                        _ => reauth.set(false),
                                    }
                                }
                            }
                        }
                        li {
                            "Search domains"
                            // TODO add support for multiple domains
                            input {
                                value: "{search_domains.read()[0]}",
                                // TODO add support for multiple domains
                                placeholder: "",
                                // TODO add support for multiple domains
                                class: "settings-form-input",
                                // TODO add support for multiple domains
                                oninput: move |e| { search_domains.set(vec![e.value.clone()]) }
                            }
                        }
                        li {
                            "Default route"
                            input {
                                r#type: "checkbox",
                                checked: if **default_route { "true" } else { "false" },
                                oninput: move |e| {
                                    match e.value.as_str() {
                                        "true" => default_route.set(true),
                                        "false" => default_route.set(false),
                                        _ => default_route.set(false),
                                    }
                                }
                            }
                        }
                        li {
                            "No routing"
                            input {
                                r#type: "checkbox",
                                checked: if **no_routing { "true" } else { "false" },
                                oninput: move |e| {
                                    match e.value.as_str() {
                                        "true" => no_routing.set(true),
                                        "false" => no_routing.set(false),
                                        _ => no_routing.set(false),
                                    }
                                }
                            }
                        }
                        li {
                            "No DNS"
                            input {
                                r#type: "checkbox",
                                checked: if **no_dns { "true" } else { "false" },
                                oninput: move |e| {
                                    match e.value.as_str() {
                                        "true" => no_dns.set(true),
                                        "false" => no_dns.set(false),
                                        _ => no_dns.set(false),
                                    }
                                }
                            }
                        }
                        li {
                            "No cert check"
                            input {
                                r#type: "checkbox",
                                checked: if **no_cert_check { "true" } else { "false" },
                                oninput: move |e| {
                                    match e.value.as_str() {
                                        "true" => no_cert_check.set(true),
                                        "false" => no_cert_check.set(false),
                                        _ => no_cert_check.set(false),
                                    }
                                }
                            }
                        }
                        li {
                            "Tunnel type"
                            select {
                                value: {
    match tunnel_type.read().to_owned() {
        TunnelType::Ssl => "SSL",
        TunnelType::Ipsec => "IPSec",
    }
},
                                onchange: move |selection| {
                                    let tunnel = match selection.data.value.clone().as_str() {
                                        "SSL" => TunnelType::Ssl,
                                        "IPSec" => TunnelType::Ipsec,
                                        _ => TunnelType::Ssl,
                                    };
                                    tunnel_type.set(tunnel);
                                },
                                option { TunnelType::Ssl.to_string() }
                                option { TunnelType::Ipsec.to_string() }
                            }
                        }
                        li {
                            "CA cert path"
                            input {
                                placeholder: "path",
                                class: "settings-form-input",
                                // TODO bound to model
                                value: "",
                                oninput: move |e| { ca_cert.set(Some(PathBuf::from(e.value.clone()))) }
                            }
                        }
                        li {
                            "Login type"
                            select {
                                value: {
    match login_type.read().to_owned() {
        model::LoginType::Password => "Password",
        model::LoginType::PasswordWithMfa => "Password with MFA",
        model::LoginType::PasswordWithMsAuth => "Password with MS auth",
        model::LoginType::EmergencyAccess => "Emergency access",
        model::LoginType::SsoAzure => "SSO Azure",
    }
},
                                onchange: move |selection| {
                                    login_type
                                        .set(
                                            match selection.data.value.clone().as_str() {
                                                "Password" => model::LoginType::Password,
                                                "Password with MFA" => model::LoginType::PasswordWithMfa,
                                                "Password with MS auth" => model::LoginType::PasswordWithMsAuth,
                                                "Emergency access" => model::LoginType::EmergencyAccess,
                                                "SSO Azure" => model::LoginType::SsoAzure,
                                                _ => model::LoginType::Password,
                                            },
                                        );
                                },
                                option { "Password" }
                                option { "Password with MFA" }
                                option { "Password with MS auth" }
                                option { "Emergency access" }
                                option { "SSO Azure" }
                            }
                        }
                    }
                }
            }
        })
    }

    fn status_service(
        cx: Scope<UserConfig>,
        status: &UseState<bool>,
        connection_status: &UseState<bool>,
    ) {
        use_coroutine(cx, |_rx: UnboundedReceiver<bool>| {
            info!("Status service coroutine called");
            let sync_status = status.to_owned();
            let connection_sync_status = connection_status.to_owned();
            async move {
                loop {
                    let socket_opt = create_client_socket();
                    socket_opt
                        .map(|socket| get_status(socket))
                        .map(|r| match r {
                            Err(e) => {
                                error!("error {}", e.to_string().as_str());
                                sync_status.set(false);
                            }
                            Ok(connected) => {
                                connection_sync_status.set(connected);
                                sync_status.set(true);
                            }
                        });
                    sleep(Duration::from_secs(5)).await;
                }
            }
        });
    }

    fn get_status(socket: UdpSocket) -> Result<bool, std::io::Error> {
        info!("Getting status");
        let _request = socket.send("\"GetStatus\"".to_string().as_bytes());
        let response_result = handle_response(socket);
        let response = match response_result {
            Ok(r) => r,
            Err(e) => return Err(e),
        };
        // TODO add parsing of the response
        let response_object: model::TunnelServiceResponse = match serde_json::from_str(&response) {
            Ok(r) => r,
            Err(_) => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "cannot parse connection status".to_string(),
                ))
            }
        };
        match response_object {
            model::TunnelServiceResponse::ConnectionStatus(status) => {
                info!("Connection status: {:?}", status);
                return Ok(status.connected_since.is_some());
            }
            model::TunnelServiceResponse::Ok => {
                info!("Connection status: Ok");
                return Ok(true);
            }
            model::TunnelServiceResponse::Error(error) => {
                error!("Connection status: Error {:?}", error);
                return Ok(false);
            }
        }
    }

    fn handle_response(socket: UdpSocket) -> Result<String, std::io::Error> {
        info!("Handling response");
        let mut buf = vec![0u8; MAX_PACKET_SIZE];
        let response = socket.recv_from(&mut buf);
        if response.is_err() || response.as_ref().unwrap().0 <= 0 {
            error!("Response not received");
            return Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "Response not received",
            ));
        }
        info!("received data: {}", response.as_ref().unwrap().0);
        // TODO add commands models
        let response_string =
            match String::from_utf8(buf.iter().cloned().filter(|&b| b != 0).collect()) {
                Ok(s) => s,
                Err(e) => format!("Invalid UTF-8 sequence: {}", e),
            };
        info!("Response: {}", response_string);
        Ok(response_string)
    }

    fn create_client_socket() -> Option<UdpSocket> {
        info!("Creating client socket");
        let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
        info!("Connecting to address {}", SERVER_ADDRESS);
        socket
            .connect(SERVER_ADDRESS)
            .expect("could not connect to adress");
        socket
            .set_read_timeout(Some(Duration::from_millis(200)))
            .expect("could not set timeout");
        socket
            .set_write_timeout(Some(Duration::from_millis(200)))
            .expect("could not set timeout");
        return Some(socket);
    }

    fn connect(params: TunnelParams) {
        info!("Connecting user to server...");
        let params_json = serde_json::to_string(&params).unwrap();
        let socket_opt = create_client_socket();
        let socket = match socket_opt {
            Some(s) => s,
            None => return,
        };
        let _request = socket.send(format!("{{\"Connect\": {}}}", params_json).as_bytes());
        let response = handle_response(socket);
        match response {
            Ok(r) => {
                info!("Response: {}", r);
            }
            Err(e) => {
                error!("Error: {}", e.to_string());
            }
        }
    }

    fn disconnect() -> () {
        info!("Disconnecting user from server...");
        let socket_opt = create_client_socket();
        let socket = match socket_opt {
            Some(s) => s,
            None => return,
        };
        let _request = socket.send("\"Disconnect\"".to_string().as_bytes());
        let _response = handle_response(socket);
    }

    fn save_config(config: UserConfig) -> Result<(), Error> {
        info!("Saving config");
        let mut file = File::create("user-config.json").unwrap();
        let res = match config.remember_me {
            true => file.write_all(
                serde_json::to_string(&UserConfig {
                    tunnel_params: remove_password(config.tunnel_params),
                    remember_me: config.remember_me,
                })
                .unwrap()
                .as_bytes(),
            ),
            false => file.write_all(
                serde_json::to_string(&UserConfig {
                    tunnel_params: TunnelParams::default(),
                    remember_me: false,
                })
                .unwrap()
                .as_bytes(),
            ),
        };
        res
    }

    fn read_config() -> Option<UserConfig> {
        info!("Reading config");
        if fs::metadata(&USER_CONF_PATH).is_err() {
            info!("Config file not found");
            return None;
        }
        let file = File::open("user-config.json").unwrap();
        let reader = std::io::BufReader::new(file);
        let params: UserConfig = serde_json::from_reader(reader).unwrap();
        return Some(params);
    }

    fn remove_password(params: TunnelParams) -> TunnelParams {
        debug!("Removing password");
        let mut params = params.clone();
        params.password = "".to_string();
        params
    }
}
