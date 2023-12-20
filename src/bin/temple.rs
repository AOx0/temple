use anyhow::Result;
use std::{process::ExitCode, str::FromStr};
use temple::Values;

fn app() -> Result<()> {
    let config = r#"
        hola = 4. 
        edades = [ [ 1, 2], 3 ] 
        nombre = "Pedro"
        objeto: { 
            nombre: "Daniel", 
            edad: 21
        }
        objetos: [
            { nombre: "David" },
            { nombre: "Daniel" },
        ]    
    "#;

    let Values(config) = Values::from_str(config)?;

    println!("{:?}", config);

    println!("{:?}", config.keys().collect::<Vec<_>>());

    Ok(())
}

fn main() -> ExitCode {
    match app() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
