[[group(0), binding(0)]]
var<storage, read> a : array<f32, 256>;
[[group(0), binding(1)]]
var<storage, read> i : u32;
[[group(0), binding(2)]]
var<storage, write> v : f32;

[[stage(compute), workgroup_size(1, 1, 1)]]
fn main()
{
    v = a[i];
}