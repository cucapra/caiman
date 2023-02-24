all:
	test

clean:
	find . -type f -name '*.out' -delete

build:
	turnt */*_baseline.cair -v --save

test:
	turnt */*_test.cair
