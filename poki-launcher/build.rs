use std::process::Command;

fn qmake_query(var: &str) -> String {
    String::from_utf8(
        Command::new("qmake")
            .env("QT_SELECT", "qt5")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    )
    .expect("UTF-8 conversion failed")
}

fn main() {
    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_library_path = qmake_query("QT_INSTALL_LIBS");
    let mut config = cpp_build::Config::new();

    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }

    config.include(qt_include_path.trim()).build("src/main.rs");

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    println!(
        "cargo:rustc-link-search{}={}",
        macos_lib_search,
        qt_library_path.trim()
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Widgets",
        macos_lib_search, macos_lib_framework
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Gui",
        macos_lib_search, macos_lib_framework
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Core",
        macos_lib_search, macos_lib_framework
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Quick",
        macos_lib_search, macos_lib_framework
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Qml",
        macos_lib_search, macos_lib_framework
    );
}
