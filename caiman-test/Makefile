all:
	test

clean:
	find . -type f -name '*.out' -delete

build:
	turnt */*_baseline.cair -v --save

test:
	test_cair
	test_ron

test_cair:
	turnt $(dir)/*_test.cair

test_ron:
	turnt $(dir)/*_test.ron
