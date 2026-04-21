mod wm;

fn main() -> anyhow::Result<()> {
    let json_str = wm::sway::info();
    if let Ok(res) = json_str {
        println!("{}", &res);
    };
    Ok(())
}
