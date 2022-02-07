#!/bin/python3 

import requests
import time
import json

HOST = "http://ma.speedtest.rcn.net/"

# jquery webapp generates a cachebuster parameter in this way
cachebuster = lambda: str(int(time.time()))

# Get modem parameters
r = requests.get(HOST + "lookup_mip_merlin-new.cgi", params={"_": cachebuster()})
modem = r.json()
print(modem)
raise SystemExit

# Get mapping of mac address to IF (of CMTS? Could I hit other modems?)
r = requests.get(HOST + "merlin/macmap_Ver2.cgi", params={"_": cachebuster(), "MAC": modem["modem"].strip(":")})
macmap = r.json()


r = requests.get(HOST + "merlin/rfmodem_us_Ver2.cgi", params = {"_" : cachebuster(), "mac": modem["modem"].strip(":"), "INT": macmap["macmapData"]["CableIF"]})

us = r.json()

good = sum([i["xGood"] for i in us["upstreamData"]])
uncorr = sum([i["yUncorr"] for i in us["upstreamData"]])

print(good)
print(uncorr)

print(uncorr / good)



