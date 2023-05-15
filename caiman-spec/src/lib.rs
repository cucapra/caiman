#![allow(warnings)]

pub mod content;
pub mod spec;
mod spec_builder;

#[cfg(test)]
mod test {
    #[test]
    fn test_build() {
        let specification = crate::content::build_spec();
    }
}
