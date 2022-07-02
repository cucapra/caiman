pub mod spec;
mod spec_builder;
pub mod content;

#[cfg(test)]
mod test
{
	#[test]
	fn test_build()
	{
		let specification = crate::content::build_spec();
	}
}