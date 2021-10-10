set terminal jpeg
set output "http-quick.jpg"

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

set title "WMS requests"
plot for [i=0:*] "results_http_quick.csv" index i using 3:18 \
  with linespoints title columnheader(2)

set output "http-quick-errors.jpg"
set title "Request errors"
set ylabel "Read errors"
set y2tics
set y2label "Status errors"

plot for [i=0:*] "results_http_quick.csv" index i \
  using 3:20 with linespoints title columnheader(2), \
    for [i=0:*] "results_http_quick.csv" index i \
    using 3:22 with linespoints title "status"
