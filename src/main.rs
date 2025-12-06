mod render;

fn main() -> anyhow::Result<()> {
    render::run()?;
    Ok(())
}
