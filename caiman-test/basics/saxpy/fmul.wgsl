[[group(0), binding(0)]]
var<storage, read> a : f32;
[[group(0), binding(1)]]
var<storage, read> b : f32;
[[group(0), binding(2)]]
var<storage, write> c : f32;

[[stage(compute), workgroup_size(1, 1, 1)]]
fn main()
{
    c = a*b;
}