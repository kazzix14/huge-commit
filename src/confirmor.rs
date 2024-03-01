pub struct Confirmor {
    assume_yes: bool,
}

impl Confirmor {
    pub fn new(assume_yes: bool) -> anyhow::Result<Self> {
        Ok(Confirmor { assume_yes })
    }

    pub fn confirm(&self, message: &'static str, default: bool) -> bool {
        if self.assume_yes {
            return true;
        } else {
            let confirm = inquire::Confirm::new(message)
                .with_default(default)
                .prompt()
                .expect("Failed to get user input");
            println!();

            confirm
        }
    }
}
