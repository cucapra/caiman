These are tested using turnt.
Simply run [ turnt *.ron ]!
(you can install turnt using pip or flit)

Also, make sure to build first using
[ cargo build --features="build-binary" ]

---------------------------------------------
This is unrelated but I just ran into a hilarious bug
where the pretty printer was printing funclets in a
nondeterministic order! It's because the RON file is
just read in whatever order I suppose and the resulting
HashMap was getting iterated through in the order that 
the file was read! I fixed it tho :)
