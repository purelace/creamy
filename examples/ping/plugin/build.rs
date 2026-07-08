use std::process::Command;

use creamy_libgen_rs::{Args, RustGen, creamy_libgen::Codegen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    //{
    //    let content = std::fs::read_to_string("protocols/success.xml")?;
    //    let protocol = compile(&mut pool, &content).unwrap();
    //    let mut outdir = outdir.clone();
    //    outdir.push("/");
    //    outdir.push(protocol.name().resolve(&pool));
    //    outdir.push(".rs");
    //    let mut file = std::fs::File::create(&outdir)?;
    //    generate(&mut file, Args::default(), &pool, &protocol)?;
    //}

    let content = std::fs::read_to_string("../definitions/ping.xml")?;

    let mut codegen = Codegen::new(&content);
    let mut outdir = outdir.clone();
    outdir.push("/");
    outdir.push(codegen.protocol_name());
    outdir.push(".rs");
    let mut rs_gen = RustGen::new(Args::default(), std::fs::File::create(&outdir)?);

    codegen.run(&mut rs_gen)?;
    let _ = Command::new("rustfmt").arg(outdir).status();

    //println!("cargo:rerun-if-changed=protocols/success.xml");
    println!("cargo:rerun-if-changed=../definitions/ping.xml");
    Ok(())
}
