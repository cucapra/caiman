use caiman_frontend::parser;

fn main() 
{
    let v = vec![
        "CPU foo : I32 -> I32;",
        "Pipeline p (po);",
        "ValueFunction foo (bar) : I32 ->I32;",
        "ValueFunction foo :I32-> I32;",
        "CPU bar : (I32, I32, I32) -> (I32);",
        "Funclet foo:I32->I32{\n hi = Phi 1; \nReturn g \n}",
        "Inline Funclet barr:I32->I32{\n hi = Phi 1; bye= Phi 2; \nReturn g, h, asdfasdfasdfasdf \n}",
        "Funclet foo:I32->I32{\n hi = Phi 1; \nYield g {h => i, j, k}\n}",
    ];
    for s in v.iter() 
    {
        let decl = parser::DeclarationParser::new()
            .parse(s)
            .unwrap();
        println!("{:?}", decl);
    }
}
