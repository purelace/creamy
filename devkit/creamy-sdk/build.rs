use creamy_libgen_rs::{Args, generate};
use creamy_xmlc::{StringPoolResolver, compile, utils::strpool::StringPool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    let mut pool = StringPool::default();

    {
        let content = std::fs::read_to_string("system.xml")?;
        let protocol = match compile(&mut pool, &content) {
            Ok(protocol) => protocol,
            Err(diag) => {
                diag.print(&content);
                panic!("Error");
            }
        };

        let mut outdir = outdir.clone();
        outdir.push("/");
        outdir.push(protocol.name().resolve(&pool));
        outdir.push(".rs");
        let mut file = std::fs::File::create(&outdir)?;
        generate(
            &mut file,
            Args {
                eq: false,
                ord: false,
                hash: false,
                creamy_sdk_path: "crate".into(),
            },
            &pool,
            &protocol,
        )?;
    }

    println!("cargo:rerun-if-changed=system.xml");
    Ok(())
}
