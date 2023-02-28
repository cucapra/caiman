all:
	test

clean:
	find . -type f -name '*.out' -delete

build:
	turnt */*_baseline.cair -v --save

test:
	turnt $(wildcard $(dir)/*_test.cair) $(wildcard $(dir)/*_test.ron) -j $(print)
