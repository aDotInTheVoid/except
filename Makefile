rfc-index.txt:
	curl -O https://www.rfc-editor.org/rfc-index.txt

rfc-numbers.txt: rfc-index.txt
	cat $< | egrep -o '^[0-9]{4}' > $@

.PHONY: download
download: rfc-numbers.txt
	parallel ./download.sh <  rfc-numbers.txt 

.PHONY: clean
clean:
	rm -rf *.txt rfcs
