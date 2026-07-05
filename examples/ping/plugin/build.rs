use creamy_libgen_rs::{Args, generate};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{StringPoolResolver, compile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    let mut pool = StringPool::default();

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

    {
        let content = std::fs::read_to_string("../definitions/ping.xml")?;
        let protocol = compile(&mut pool, &content).unwrap();
        let mut outdir = outdir.clone();
        outdir.push("/");
        outdir.push(protocol.name().resolve(&pool));
        outdir.push(".rs");
        let mut file = std::fs::File::create(&outdir)?;
        generate(&mut file, Args::default(), &pool, &protocol)?;
    }

    println!("cargo:rerun-if-changed=protocols/success.xml");
    println!("cargo:rerun-if-changed=../definitions/ping.xml");
    Ok(())
}
