[[group(0), binding(0)]]
var<storage, read> a : array<f32, 256>;
[[group(0), binding(1)]]
var<storage, read> i : u64;
[[group(0), binding(2)]]
var<storage, read> v : f32;
[[group(0), binding(3)]]
var<storage, write> mutated : array<f32, 256>;

[[stage(compute), workgroup_size(1, 1, 1)]]
fn main()
{
    // This is SO AWFUL since I can't in-place mutate the array
    for(var j = 0; j < 256; j++){
        mutated[j] = a[j];
    }
    mutated[i] = v;
}