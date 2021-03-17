-- Resource: https://github.com/timotta/wrk-scripts/blob/master/multiplepaths.lua

-- Initialize the pseudo random number generator
-- Resource: http://lua-users.org/wiki/MathLibraryTutorial
math.randomseed(os.time())
math.random(); math.random(); math.random()

-- Shuffle array
-- Returns a randomly shuffled array
function shuffle(paths)
  local j, k
  local n = #paths

  for i = 1, n do
    j, k = math.random(n), math.random(n)
    paths[j], paths[k] = paths[k], paths[j]
  end

  return paths
end

-- Load URL paths from the file
function load_url_paths_from_file(file)
  lines = {}

  -- Check if the file exists
  -- Resource: http://stackoverflow.com/a/4991602/325852
  local f=io.open(file,"r")
  if f~=nil then 
    io.close(f)
  else
    -- Return the empty array
    return lines
  end

  -- If the file exists loop through all its lines 
  -- and add them into the lines array
  for line in io.lines(file) do
    if not (line == '') then
      lines[#lines + 1] = line
    end
  end

  return shuffle(lines)
end

-- Load URL paths from file
paths = load_url_paths_from_file("/paths.txt")

-- Check if at least one path was found in the file
if #paths <= 0 then
  print("multiplepaths: No paths found. You have to create a file paths.txt with one path per line")
  os.exit()
end

--print("multiplepaths: Found " .. #paths .. " paths")

-- Initialize the paths array iterator
counter = 0

request = function()
  -- Get the next paths array element
  url_path = paths[counter]

  -- 
  counter = counter + 1

  -- If the counter is longer than the paths array length then reset it
  if counter > #paths then
    counter = 0
  end

  -- Return the request object with the current URL path
  return wrk.format(nil, url_path)
end


done = function(summary, latency, requests)
  -- open output file
  local fn = "/bench/results/results_" .. os.getenv("CSV_NAME")
  local f = io.open(fn,"r")
  local newcsv = (f==nil)
  if f~=nil then
    io.close(f)
  end

  f = io.open(fn, "a+")
  if newcsv then
    f:write("#time_started,software,connections,min_requests,max_requests,mean_requests,min_latency,max_latency,mean_latency,stdev,50th,90th,99th,99.999th,duration,requests,bytes,request_per_sec,connect_errors,read_errors,write_errors,status_errors,timeouts\n")
  end
  f:write(string.format("%s,%s,%d,%f,%f,%f,%f,%f,%f,%f,%f,%f,%f,%f,%d,%d,%f,%d,%d,%d,%d,%d\n",
    os.date("!%Y-%m-%dT%TZ"),
    os.getenv("SW"),
    os.getenv("CONNECTIONS"),
    requests.min,   -- per-thread request rate
    requests.max,
    requests.mean,
    latency.min,    -- per-request latency
    latency.max,
    latency.mean,
    latency.stdev,

    latency:percentile(50),     -- 50percentile latency
    latency:percentile(90),     -- 90percentile latency
    latency:percentile(99),     -- 99percentile latency
    latency:percentile(99.999), -- 99.999percentile latency

    summary["duration"],          -- duration of the benchmark
    summary["requests"],          -- total requests during the benchmark
    summary["bytes"],             -- total received bytes during the benchmark

    summary["requests"]/(summary["duration"]/1000000), -- Total requests/sec

    summary["errors"]["connect"], -- total socket connection errors
    summary["errors"]["read"],    -- total socket read errors
    summary["errors"]["write"],   -- total socket write errors
    summary["errors"]["status"],  -- total socket write errors
    summary["errors"]["timeout"]  -- total request timeouts
    ))

  f:close()
end
