mod render;

fn main() -> anyhow::Result<()> {
    render::state::run()?;
    Ok(())
}
