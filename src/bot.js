
const cloudscraper = require('cloudscraper');
// const cloudscraper = require('cloudflare-scraper');
// const Humanoid = require('humanoid-js');
var HTMLParser = require('node-html-parser');
const dom = require('node-html-parser').parse;
// Using require to access url module 
const url = require('url');

// const cloudscraper = new Humanoid();
// cloudscraper.defaultParams.pool = {
//     maxSockets: 128
// };
// cloudscraper.forever = true;

const DISCARD_BACKLOG = new Map();

const OUTDATED_ORDERS_AGE = 20e3;
const CLEANUP_INTERVAL = 90e3;

const DEBUG = true;


function delay(t) {
    return new Promise((r, e) => setTimeout(r, t));
}

function log() {
    if(DEBUG){
        console.log(...arguments);
    }
}

function do_login() {
    let url = 'https://essayshark.com/auth/aj_login2.html?callback=';
    let payload = {
        l: 'cmutungi17@yahoo.com',
        p: 'Log@nj@b@li2020',
        stay_signed_in: 1,
        marketing: 0,
        policy: 0,
        role: '',
    };

    return cloudscraper.post(url, {
        formData: payload
    }).then((r) => {
        log('Login done', r);
        return true;
    }).catch(err => {
        log('Login err', err);
        return false;
    });

}


function fail(cb, action=null, order=null, ...args) {
    if(typeof cb === 'string')
        order === null || args.unshift(order),
        order = action,
        action = cb,
        cb = Promise.resolve();
    else if(typeof cb === 'function')
        cb = Promise.resolve(action());
    else
        cb = Promise.resolve(cb);

    let msg = order ?  `Error while ${action} order [${order.id}]:` : `Error while ${action}:`;
    // order && DISCARD_BACKLOG.set(order.id, DISCARD_BACKLOG.get(order.id) || Date.now());
    log(msg, ...args);    

    return cb;
}



function discard_outdated_orders(all=false) {
    let now = Date.now(), 
        maxage = OUTDATED_ORDERS_AGE;

    const outdated = {};

    // DISCARD_BACKLOG.forEach(o => (now - o[0]) > age && (outdated[o[1].id] = o[0]));

    // log(`Cleanup...`);

    // if (all === true){
    //     DISCARD_BACKLOG.clear();
    // } else {
    //     let n = 0;
    //     for (const [id, t] of [...DISCARD_BACKLOG]) {
    //         if((now - t) < maxage)
    //             continue;
    //         n++;
    //         outdated[id] = t;
    //         DISCARD_BACKLOG.delete(id);
    //     }
    //     if(!n)
    //         return Promise.resolve();
    // }
    
    return get_orders({ all: true }).then(list => {

        let orders = all === true ? list.filter(o => outdated[o.id]) : list;

        log(`discarding orders [${orders.length}]`, orders.map(o => ({ 
            id: o.id, 
            age: `${((now - (outdated[o.id] || 0))/1e3).toFixed(3)} secs`, 
            order_read: o.order_read, 
            bid_outdated: o.bid_outdated 
        })));

        if(orders.length)
            return cloudscraper.post(`https://essayshark.com/writer/orders/aj_source.html`, {
                formData: {
                    act: 'discard_all',
                    nobreath: 1,
                    ids: orders.map(o => o.id).join(', ')
                }
            });
    });
}



function submit_bid(order) {
    // if(active_orders[order.id])
        // return;
    
    // active_orders[order.id] = true;
    let formData = {
        'bid_add_ua': 'mmmmmm',
        'bid_add': 1,
        'bid': +order.min_price_total / +order.pages_qty,
    };

    return cloudscraper.post(
        `https://essayshark.com/writer/orders/${order.id}.html`, 
        { formData },
        // order.bidPostData,
        (e, r, b) => {
            if(e) 
                return fail('submitting bid', order, e);
            
            // delay().then(() => (delete active_orders[order.id]));

            log(`bid submitted [${order.id}]: in total ${((Date.now() - order.received_at) / 1000).toFixed(4)} secs`);
    });
}



function download_file(order, b, n=1) {

    if (order.downloaded)
        return;

    order.downloaded = true;

    // let res = cloudscraper.get(`https://essayshark.com/writer/orders/${order.id}.html`); // Promise.resolve(b);
    let res = Promise.resolve(b);
    
    return !n ? res : res.then(((body) => {
            let html = HTMLParser.parse(body),
                el = html.querySelector('a[target="download_ifm"][href]');

            if(el){
                let url = new URL(el.getAttribute('href'), `https://essayshark.com/`);
                log(`Downloading File [${order.id}]: ${el.innerText} (${url.href}) ...`);
                return cloudscraper.get(url.href)
                    // .then(() => log(`Download DONE [${order.id}]: ${el.innerText} (${url.href})`))
                    .catch((e) => fail('downloading file', order, `${el.innerText} (${url.href})`, e));
            }
            
        }));
}

const pings = {};

function queue_bid(order, res, x=100) {
    cloudscraper.get(`https://essayshark.com/writer/orders/ping.html?order=${order.id}&_=`, (e, r, b) => {
        if(e)
            return fail('queueing', order, e);

        let data = pings[b] || (pings[b] = JSON.parse(b)),
            ttr = data.read_time_remain;
        
        if(ttr == 0) {
            return submit_bid(order);
        } else if(ttr == 10) {
            if (data.pr) {
                submit_bid(order);
                return cloudscraper.get(`https://essayshark.com/writer/orders/ping.html?order=${order.id}&_=`)
                            .catch((e) => fail('queueing', order, e));
            }
            else if(x < 1)
                return queue_bid(order, res);
            else
                return delay(x).then(() =>  queue_bid(order, res, x-10));
        } 

        delay((ttr - 10) > 10 ? 2000 : 800).then(() => queue_bid(order, res));
        data.files_download_remain && download_file(order, res);
    });
}

function dispatch_order(order) {

    const res = cloudscraper.get(`https://essayshark.com/writer/orders/${order.id}.html`);
    
    queue_bid(order, res);

    order.received_at = Date.now();
    // order.bidPostData = bid_submit_options(+order.min_price_total / +order.pages_qty);

    return res;
}


const formData = {};
    

function bid_submit_options(amt) {
    let val = formData[amt];
    if(val)
        return val;
        
    var str = [];
    let form = {
        'bid_add_ua': 'mmmmmm',
        'bid_add': 1,
        'bid': amt,
    };
    for (var p in form)
      if (form.hasOwnProperty(p)) {
        str.push(encodeURIComponent(p) + "=" + encodeURIComponent(form[p]));
      }
    return formData[amt] = { form: str.join("&") };
}



function get_orders(opts) {
    let url = 'https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_=';
    if (!opts || !opts.all)
        return cloudscraper.get(url).then(r => (JSON.parse(r).new_items || []).filter(o => !(o.order_read != 'N' || o.bid_outdated == 'Y')));
    else
        return cloudscraper.get(url).then(r => JSON.parse(r).new_items || []);
}


function dispatch_orders(res) {
    
    const orders = JSON.parse(res).new_items
    
    if(!orders.length)
        return find_orders();

    const q = [];

    for (let i = 0, len=orders.length; i < len; i++) {
        const o = orders[i];
        if (!(o.order_read != 'N' || o.bid_outdated == 'Y')) 
            q.push(dispatch_order(o));

    }

    return Promise.all(q).then(find_orders);

}


function find_orders() {
    let url = 'https://essayshark.com/writer/orders/aj_source.html?act=load_list&nobreath=1&session_more_qty=0&session_discarded=0&_=';
    return cloudscraper
        .get(url)
        .then(dispatch_orders)
        .catch(e => fail(delay(10).then(find_orders), 'fetching orders', null, e));
}



 
do_login().then(r => {
    log(' ');
    if (!r) return;

    return discard_outdated_orders(true).then(find_orders);
}).then(r => {
    console.error('---xxxXXX ERRORED XXXxxx---');
    console.error(r);
    log('Exiting...');
    process.exit(1);
});

