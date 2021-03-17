set terminal jpeg
set output "http.jpg"

# Where to place the legend/key
set key left top
# Draw gridlines oriented on the y axis
set grid y
set ylabel "Requests/s"
set logscale x
set xlabel "Connections"
set xtics (1,4,32,64,128,256)

# Use CSV delimiter instead of spaces (default)
set datafile separator ','

# graph title
set title "WMS Requests"
plot for [i=0:*] "results_http.csv" index i using 3:18 \
  with linespoints title columnheader(2)
