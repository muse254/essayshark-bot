const cloudscraper = require('cloudscraper')
const fs = require('fs')

let options = {
  uri: 'https://essayshark.com/auth/aj_login2.html?callback=',
  formData: {
    l: 'cmutungi17@yahoo.com',
    p: 'Log@nj@b@li2020',
    stay_signed_in: 1,
    marketing: 0,
    policy: 0,
    role: ''
  }
}

cloudscraper.post(options, (error, response, body) => {
  if (error) {
    console.log('Login err', error)
    return
  }

  // body is wrapped in (content);, trim them out
  let json = JSON.parse(body.replace(/^\(|\)|;$/g, ''))

  // confirm login
  if (json.code == 1) {
    console.log('Login Successful')
    let cookie_str = response.headers['set-cookie'][0]
    if (cookie_str) {
      // retreive the string that comes before the ; char
      fs.writeFileSync('cookie.txt', cookie_str)
    }
    return
  }
  console.log('Login Failed')
  console.log(json)
})
