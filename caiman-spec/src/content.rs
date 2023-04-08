use crate::spec;
use crate::spec_builder;

pub fn build_spec() -> spec::Spec {
    //let mut builder = spec_builder::SpecBuilder::new();

    let content_str = include_str!("content.ron");

    let result: Result<spec::Spec, ron::de::Error> = ron::from_str(&content_str);
    match result {
        Err(why) => panic!("Parse error: {}", why),
        Ok(specification) => {
            println!("{:?}", specification);
            specification
        }
    }
}
