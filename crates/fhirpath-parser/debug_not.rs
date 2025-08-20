use octofhir_fhirpath_parser::parse; fn main() { match parse("not true") { Ok(ast) => println\!("AST: {:#?}", ast), Err(e) => println\!("Error: {:?}", e) } }
