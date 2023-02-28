all:
	test

clean:
	find . -type f -name '*.log' -delete
	find . -type f -name '*.debug' -delete

build:
	turnt */*_baseline.cair -v --save

test:
	turnt $(wildcard $(dir)/*_test.cair) $(wildcard $(dir)/*_test.ron) $(print) $(diff)
