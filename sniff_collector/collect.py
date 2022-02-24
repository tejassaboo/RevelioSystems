#!/usr/bin/env python3

from scapy.all import sniff, IP, TCP
from scapy.layers.http import HTTPRequest, HTTPResponse
from Crypto import Random
import os
import queue
import threading
import hashlib
import json
import hmac
import base64
import ipaddress
import requests
import time

ENCODING = 'utf-8'
BYTEORDER = 'big'

pkts = queue.Queue()
reqs = queue.Queue()
idMap = {}
TIMEUNIT = 1000000000

class TCPIP:
	def __init__(self, src, dst, sport, dport):
		self.src = ipaddress.ip_address(src)
		self.dst = ipaddress.ip_address(dst)
		self.sport = sport
		self.dport = dport
	
	def __hash__(self):
		return hash(self.src) ^ hash(self.dst) ^ hash(self.sport) ^ hash(self.dport)
	
	def __eq__(self, other):
		return self.src == other.src and self.dst == other.dst and self.sport == other.sport and self.dport == other.dport

def send():
	ENDPOINT = 'https://' + INFLUX_ADDR
	S = requests.Session()
	while True:
		req = reqs.get()
		S.post(ENDPOINT + '/update', json=req)


def genreq(payload):
	tag = {}
	tag['nonce'] = int.from_bytes(Random.get_random_bytes(16), byteorder=BYTEORDER)
	
	expireTime = time.time() + 60
	secs = int(expireTime)
	nanos = int((expireTime - secs) * TIMEUNIT)
	expires = {}
	expires['secs'] = secs
	expires['nanos'] = nanos
	tag['expires'] = expires
	tag['payload'] = payload
	
	message = json.dumps(tag, separators=(',', ':'))
	mac = hmac.new(INFLUX_SKEY, msg=bytes(message, ENCODING), digestmod='sha256').digest()
	sig = base64.b64encode(mac)
	
	req = {}
	req['message'] = message
	req['sig'] = str(sig, ENCODING)
	
	reqs.put(req)


def process():
	while True:
		pkt = pkts.get()
		
		if pkt.haslayer(HTTPRequest):
			reqId = pkt[HTTPRequest].X_Request_ID
			if reqId != None:
				metrics = {}
				
				metrics['time'] = round(pkt.time * TIMEUNIT)  
				metrics['method'] = pkt[HTTPRequest].Method.decode()
				metrics['uri'] = pkt[HTTPRequest].Path.decode()
				metrics['name'] = INFLUX_NAME
				metrics['gateway'] = False
				tcpip = {}
				metrics['tcpip'] = tcpip
				
				h = hashlib.md5()
				h.update(reqId)
				metrics['id'] = int.from_bytes(h.digest(), byteorder=BYTEORDER)
				
				src = pkt[IP].src
				dst = pkt[IP].dst
				sport = pkt[TCP].sport
				dport = pkt[TCP].dport
				tcpip['src'] = src
				tcpip['dst'] = dst
				tcpip['sport'] = sport
				tcpip['dport'] = dport
				
				k = TCPIP(src, dst, sport, dport)
				idMap[k] = metrics

		elif pkt.haslayer(HTTPResponse):
			src = pkt[IP].src
			dst = pkt[IP].dst
			sport = pkt[TCP].sport
			dport = pkt[TCP].dport
			k = TCPIP(dst, src, dport, sport)
			
			if k in idMap.keys():
				metrics = idMap[k] 
				del idMap[k]
				
				newTimestamp = round(pkt.time * TIMEUNIT)
				metrics['duration'] = newTimestamp - metrics['time']
				
				genreq(metrics)


def cap(pkt):
	pkts.put(pkt)


def setup():
	global INFLUX_SKEY, INFLUX_ADDR, INFLUX_IFACE, INFLUX_NAME
	
	INFLUX_SKEY = os.getenv('INFLUX_SKEY')
	if INFLUX_SKEY == None:
		print('Key not set.')
		exit(-1)
	INFLUX_SKEY = base64.b64decode(INFLUX_SKEY)

	INFLUX_ADDR = os.getenv('INFLUX_ADDR')
	if INFLUX_ADDR == None:
		print('Address not set.')
		exit(-1)

	INFLUX_IFACE = os.getenv('INFLUX_IFACE')
	if INFLUX_IFACE == None:
		print('Interface not set.')
		exit(-1)
		
	INFLUX_NAME = os.getenv('INFLUX_NAME')
	if INFLUX_NAME == None:
		print('Database not set.')
		exit(-1)


setup()
threading.Thread(target=send, daemon=True).start()
threading.Thread(target=process, daemon=True).start()

sniff(filter='tcp', prn=cap, iface=INFLUX_IFACE, store=False)
