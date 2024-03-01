pub struct Confirmor {}

impl Confirmor {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Confirmor {})
    }

    pub fn confirm(&self, message: &'static str, default: bool) -> bool {
        let confirm = inquire::Confirm::new(message)
            .with_default(default)
            .prompt()
            .expect("Failed to get user input");
        println!();

        confirm
    }
}
