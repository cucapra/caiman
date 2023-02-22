[[group(0), binding(0)]]
var<storage, read_write> a : f32;
[[group(0), binding(1)]]
var<storage, read> b : f32;
[[stage(compute), workgroup_size(1, 1, 1)]]
fn main() // must be called main for now!
{
    a = a*b;
}