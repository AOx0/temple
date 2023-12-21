use anyhow::Result;
use std::{process::ExitCode, str::FromStr};
use temple::values::{Type, Values};

fn app() -> Result<()> {
    let config = r#"
        edad: Number
        edad2: Any
        hola: Number = 4.;
        vacio = [];
        edades = [ 1, 2 , 3 ] 
        nombre = "Pedro"
        objeto = { 
            nombre: "Daniel", 
            edad: 21
        }
        objetos = [
            { nombre: "David" },
            { nombre: "Daniel" },
        ]    
    "#;

    let Values {
        value_map: config,
        type_map: types,
    } = Values::from_str(config)?;

    println!("{:?}", config);

    println!("{:?}", config.keys().collect::<Vec<_>>());

    for (k, v) in config.iter() {
        println!(
            "Type of {k}: {t} (declared: {d})",
            t = Type::from(v),
            d = types.get(k).unwrap()
        );
    }

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
