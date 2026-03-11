use clap::CommandFactory;

fn main() {
    let cmd = posthog::cli::Cli::command();
    let man_dir = std::path::Path::new("man");
    std::fs::create_dir_all(man_dir).expect("Failed to create man directory");
    clap_mangen::generate_to(cmd, man_dir).expect("Failed to generate man pages");
    println!("Man pages generated in {}", man_dir.display());
}
