use static_files::NpmBuild;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
fn get_git_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let child = Command::new("git").args(["describe", "--always"]).output();
    match child {
        Ok(child) => {
            let buf = String::from_utf8(child.stdout).expect("failed to read stdout");
            version + "#" + &buf
        }
        Err(err) => {
            eprintln!("`git describe` err: {}", err);
            version
        }
    }
}
fn main() -> std::io::Result<()> {
    let version = get_git_version();
    let mut f = File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("VERSION")).unwrap();
    f.write_all(version.trim().as_bytes()).unwrap();
    if Path::new("web/dist/index.html").exists() {
        println!("cargo:warning=web/dist 已存在，跳过 npm 构建（如需重新构建请运行 make build-ui）");
    } else {
        println!("cargo:warning=web/dist 不存在，执行 npm install && npm run build（可先运行 make build-ui 预构建以解耦 Node 工具链）");
        let npm = NpmBuild::new("web").install()?.run("build")?;
        let _npm = npm.change_detection();
    }
    static_files::resource_dir("web/dist").build()
}
