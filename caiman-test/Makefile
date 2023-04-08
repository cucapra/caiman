all:
	test

clean:
	find . -type f -name '*.log' -delete
	find . -type f -name '*.debug' -delete

explicate_setup:
	turnt $(wildcard */*_baseline.cair) -v --save

test:
	turnt $(wildcard */*_test.cair) $(wildcard */*_test.ron) $(print) $(diff)

test_dir:
	turnt $(wildcard $(dir)/*_test.cair) $(wildcard $(dir)/*_test.ron) $(print) $(diff)