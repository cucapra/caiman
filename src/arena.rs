use std::collections::HashMap;
use std::default::Default;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arena<T>
{
	elements : HashMap<usize, T>,
	unused_ids : Vec<usize>,
	next_id : usize
}

impl<T> Arena<T>
{
	pub fn new() -> Self
	{
		let mut unused_ids = Vec::<usize>::new();
		let mut next_id : usize = 0;
		Self{elements : HashMap::<usize, T>::new(), unused_ids, next_id}
	}

	pub fn from_hash_map(elements : HashMap<usize, T>) -> Self
	{
		let mut unused_ids = Vec::<usize>::new();
		let mut next_id : usize = 0;
		let mut element_count = elements.len();
		while element_count > 0
		{
			// This might have a bug
			if elements.contains_key(& next_id)
			{
				element_count -= 1;
			}
			else
			{
				unused_ids.push(next_id);
			}
			next_id += 1;
		}
		Self{elements, unused_ids, next_id}
	}

	pub fn get_next_id(&self) -> usize {
		self.next_id
	}

	fn pop_unused_id(&mut self) -> usize
	{
		if let Some(id) = self.unused_ids.pop()
		{
			return id;
		}

		let id = self.next_id;
		self.next_id += 1;
		return id;
	}

	pub fn create(&mut self, value : T) -> usize
	{
		let id = self.pop_unused_id();
		// Should check if there are no collisions for debugging
		self.elements.insert(id, value);
		id
	}

	pub fn iter<'m> (& 'm self) -> Iterator<'m, T>
	{
		Iterator::<'m, T>{iter : self.elements.iter()}
	}

	pub fn iter_mut<'m> (& 'm mut self) -> IteratorMut<'m, T>
	{
		IteratorMut::<'m, T>{iter : self.elements.iter_mut()}
	}

	pub fn get<'m> (& 'm self, index : &usize) -> std::option::Option<&T> {
		self.elements.get(index)
	}
}

impl <T> Default for Arena<T>
{
	fn default() -> Self
	{
		Self::from_hash_map(Default::default())
	}
}

impl<T> core::ops::Index<&usize> for Arena<T>
{
	type Output = T;
	fn index(&self, index: &usize) -> &Self::Output
	{
		& self.elements.index(index)
	}
}

impl<T> core::ops::IndexMut<&usize> for Arena<T>
{
	fn index_mut(&mut self, index: &usize) -> &mut Self::Output
	{
		self.elements.get_mut(index).unwrap()
	}
}

impl<T> Serialize for Arena<T>
	where T : Serialize
{
	fn serialize<S>(& self, serializer : S) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
		where S : Serializer
	{
		self.elements.serialize(serializer)
	}
}

impl<'de, T> Deserialize<'de> for Arena<T>
	where T : Deserialize<'de>
{
	fn deserialize<D>(deserializer : D) -> std::result::Result<Self, <D as Deserializer<'de>>::Error>
		where D : Deserializer<'de>
	{
		let elements = HashMap::<usize, T>::deserialize(deserializer)?;
		Ok(Self::from_hash_map(elements))
	}
}

pub struct Iterator<'m, T>
{
	iter : std::collections::hash_map::Iter<'m, usize, T>
}

impl<'m, T> std::iter::Iterator for Iterator<'m, T>
{
	type Item = (& 'm usize, & 'm T);

	fn next(&mut self) -> Option<Self::Item>
	{
		self.iter.next()
	}
}

pub struct IteratorMut<'m, T>
{
	iter : std::collections::hash_map::IterMut<'m, usize, T>
}

impl<'m, T> std::iter::Iterator for IteratorMut<'m, T>
{
	type Item = (& 'm usize, & 'm mut T);

	fn next(&mut self) -> Option<Self::Item>
	{
		self.iter.next()
	}
}