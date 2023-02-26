all:
	test

clean:
	find . -type f -name '*.out' -delete

build:
	turnt */*_baseline.cair -v --save

test_cair:
	turnt $(dir)/*_test.cair -j $(print)

test_ron:
	turnt $(dir)/*_test.ron -j $(print)
