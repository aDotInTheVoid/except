rfc-index.txt:
	curl -O https://www.rfc-editor.org/rfc-index.txt

rfc-numbers.txt: rfc-index.txt
	cat $< | grep -v 'Not Issued.' | egrep -o '^[0-9]{4}' > $@

.PHONY: download
download: rfc-numbers.txt
	mkdir -p rfcs
	parallel ./download.sh <  rfc-numbers.txt 

.PHONY: clean
clean:
	rm -rf *.txt rfcs
