# Super test script which combines everything

cargo build --all

if [ "$1" != "-t" ]; then
    python3 test.py run
    if [ $? != 0 ]; then
        exit 1
    fi
fi

find ./high-level-caiman/turnt -name "*.cm" | xargs turnt --diff -v