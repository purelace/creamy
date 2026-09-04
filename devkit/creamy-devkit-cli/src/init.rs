use std::path::Path;

pub fn init_template(workdir: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let id = loop {
        let value = rl.readline(">> ID: ")?;
        match creamy_template::Id::new(value) {
            Ok(id) => break id,
            Err(err) => {
                println!("Error: {err}");
            }
        }
    };

    let name = loop {
        let value = rl.readline(">> Name: ")?;
        if !value.is_empty() {
            break value;
        }
    };

    let desc = loop {
        let value = rl.readline(">> Description: ")?;
        if !value.is_empty() {
            break value;
        }
    };

    let repo = loop {
        let value = rl.readline(">> Repository: ")?;
        if !value.is_empty() {
            break value;
        }
    };

    creamy_template::generate_template(
        workdir,
        &creamy_template::Template {
            id,
            name,
            description: desc,
            repository: repo,
        },
    )?;
    Ok(())
}
