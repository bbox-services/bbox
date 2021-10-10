all: http-quick.jpg http.jpg

quick: http-quick.jpg

http.jpg: http.g results_http.csv
	gnuplot http.g

http-quick.jpg: http-quick.g results_http_quick.csv
	gnuplot http-quick.g

clean:
	rm -f *.csv
