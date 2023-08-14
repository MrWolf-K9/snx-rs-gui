# snx-rs-gui
Rust gui app for the [snx-rs](github.com/ancwrd1/snx-rs). Written with Dioxus gui library.

## Build
Folowing https://dioxuslabs.com/docs/0.3/guide/en/publishing/desktop.html install and run cargo-bundle --release

## Prerequisites
Have running snx-rs service in `-m command` mode

## Limitations
- Search domains support only one domain at this moment
- You need to have installed 

| Package       | Install command                               |
|---------------|-----------------------------------------------|
| libwebkit2gtk | sudo apt-get install -y libwebkit2gtk-4.1-dev |