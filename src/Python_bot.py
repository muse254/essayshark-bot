import time, validators, json, threading, logging, requests, queue
import time, validators, json, threading, logging, requests, queue
from bs4 import BeautifulSoup
import cloudscraper
import subprocess
import datetime
import re

LOGIN = 'aqademiawrita@gmail.com'
PASSWORD = 'Keny@#2030'

HEADERS = {
	"user-agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36",
	"accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9",
	"accept-language": "en-US,en;q=0.9,sw;q=0.8",
	"cache-control": "max-age=0",
	"upgrade-insecure-requests": "1"
}

def fetch_post(session, url, payload, headers=HEADERS):
	r = session.post(url, data=payload, headers=HEADERS)
	return r.text

def fetch_get(session, url):
	r = session.get(url, headers=HEADERS)
	return r.text

def discard_orders(session, order_ids):
	payload = {
		'act': 'discard_all',
		'nobreath': 1,
		'ids': ','.join(order_ids),
	}
	fetch_post(session, 'https://essayshark.com/writer/orders/aj_source.html', payload)


def get_orders(session):
	orders = []
	response = fetch_get(session, 'https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_=')
	try:
		data = json.loads(response)
	except json.decoder.JSONDecodeError as e:
		data = None
		fails = fails + 1
		print(e)

	qty_total = data['qty_total']
	if qty_total < 1:
		return orders

	items = data['new_items']
	for item in items:
		orders.append(str(item['id']))

	return orders
	
def discard_all_orders(session):
	orders = get_orders(session)
	while len(orders) > 0:
		time.sleep(0.1)
		discard_orders(session, orders)
		orders = get_orders(session)



def worker(scraper, order_id, q):	
	link = f'https://essayshark.com/writer/orders/{order_id}.html'
	print(link)
	r = scraper.get(link)
	soup = BeautifulSoup(r.content, 'html.parser')
	a = soup.find_all(attrs={"target": "download_ifm"})
	if(len(a)):
		href = a[0].get('href')
		dwnld_link = f'https://essayshark.com/{href}'
		print(dwnld_link)
		scraper.get(dwnld_link)



def order_run(scraper, order_id, bid_amount):
	with open("url.txt", "w") as f:
		f.write(f'https://essayshark.com/writer/orders/{order_id}.html\n')
		f.write(str(bid_amount))
	scraper.get(f'https://essayshark.com/writer/orders/{order_id}.html')
	print("cookies are:")
	cookie = scraper.cookies.get_dict()
	ping_url = f'https://essayshark.com/writer/orders/ping.html?order={order_id}&_='
	print(ping_url)
	# time = datetime.datetime.now()
	# time += datetime.timedelta(0,10)
	# s = subprocess.run(f"timedatectl set-time {re.split(' ', str(time))[-1]}", shell=True)
	t1 = time.time()
	try:
		j = scraper.get(ping_url, cookies = cookie).json()
		print(j)
	except json.decoder.JSONDecodeError as e:
		print(f'{order_id}: something went wrong' )
		return

	files_download_remain = j['files_download_remain']
	read_time_remain = j['read_time_remain']
	pr = j['pr']
	thread_started = False
	fails = 0
	x=0
	while True:
		try:
			j = scraper.get(f'https://essayshark.com/writer/orders/{order_id}.html')
			print("cookies are:")
			cookie = scraper.cookies.get_dict()
			j = scraper.get(ping_url).json()
			print(j)
		except json.decoder.JSONDecodeError as e:
			print(f'{order_id}: something went wrong' )
			fails = fails + 1
			if fails > 5:
				break

		fails = 0

		files_download_remain = j['files_download_remain']
		read_time_remain = j['read_time_remain']
		pr = j['pr']

		if not thread_started and files_download_remain == 1:
			q = queue.Queue()
			thread = threading.Thread(target=worker, args=(scraper,order_id,q))
			thread.start()
			thread_started = True
			print("Thread started")

		# if pr == 1 and read_time_remain == 10 and files_download_remain == 0:
		# 	print(bid_amount)
		# 	if bid_amount > 0:
		# 		# time.sleep(0.1)
		# 		payload = {
		# 			'bid_add_ua': 'mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm',
		# 			'bid_add': 1,
		# 			'bid': bid_amount,
		# 			'read_time_remain':10
		# 			}
		# 		link = f'https://essayshark.com/writer/orders/{order_id}.html'
		# 		print(link)
		# 		r = scraper.post(link, data=payload)
		# 		print(r.status_code)
		# 	else:
		# 		print("Could not find bid amount")
		# 	break
		
		if read_time_remain == 0 and files_download_remain == 0:
			print(bid_amount)
			if bid_amount > 0:
				#time.sleep(0.1)
				payload = {
					'bid_add_ua': 'mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm',
					'bid_add': 1,
					'bid': bid_amount,
					}
				link = f'https://essayshark.com/writer/orders/{order_id}.html'
				print(link)
				r = scraper.post(link, data=payload)
				print(r.status_code)
			else:
				print("Could not find bid amount")
			break

		t2 = time.time()
		print(t2 - t1)

# s = requests.Session()
url = 'https://essayshark.com/auth/aj_login2.html?callback='
payload = {
	'l': LOGIN,
	'p': PASSWORD,
	'stay_signed_in': 1,
	'marketing': 0,
	'policy': 0,
	'role': '',
		}
# r = s.post(url, data=payload)
# print(r.text)
# if r.status_code != requests.codes.ok:
# 	print("Login not allowed")
# 	exit()

import cloudscraper
scraper = cloudscraper.create_scraper()
r = scraper.post(url, data=payload)
print(r.text)
if r.status_code != requests.codes.ok:
	print("Login not allowed")
	exit()

else:
	print("Login Successful")
	time.sleep(1)
	discard_all_orders(scraper)
	time.sleep(1)
	fails = 0
	fails2 = 0
	wait_time = 0.1
	print("cookies are:")
	print(scraper.cookies.get_dict())
while True:
	url = 'https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_='
	r = scraper.get(url)
	print(r.text)
	if r.status_code != requests.codes.ok:
		fails = fails + 1
		time.sleep(0.1)

	if fails > 5:
		break

	fails = 0
	try:
		data = r.json()
		print(data)
	except json.decoder.JSONDecodeError as e:
		time.sleep(0.1)
		fails2 = fails2 + 1
		if fails2 > 5:
			break
		continue

	fails2 = 0

	items = data['new_items']
	if len(items):
		for item in items:
			order_id = item['id']
			order_read = item['order_read']
			bid_outdated = item['bid_outdated']
			min_price_total = item['min_price_total']
			pages_qty = item['pages_qty']
			print(bid_outdated)
			print(min_price_total)
			bid_amount = float(min_price_total)/float(pages_qty)
			if order_read.lower() == 'n' or (bid_outdated is not None and bid_outdated.lower() == 'y'):
				wait_time = 0.1
				order_run(scraper,order_id, bid_amount)
				#thread = threading.Thread(target=order_run, args=(s,order_id, bid_amount))
				#thread.start()

	time.sleep(0.1)
	wait_time = 0.1
			
			
	

