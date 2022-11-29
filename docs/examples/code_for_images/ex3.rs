fn value() {

    let x : i32 = 5;            
    let y : i32 = 7;
    let z1 : i32 = x + y;
    let z2 : i32 = x + y;
}

fn schedule() {
    cpu(x);
    cpu(y);
    fence(x);
    fence(y);
    cpu(z1);
    gpu(z2);
}