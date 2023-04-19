[[group(0), binding(0)]]
var<storage, read> a : u64;
[[group(0), binding(1)]]
var<storage, read> b : u64;
[[group(0), binding(2)]]
var<storage, write> c : u64;

[[stage(compute), workgroup_size(1, 1, 1)]]
fn main()
{
    c = a < b ? 1 : 0;
}