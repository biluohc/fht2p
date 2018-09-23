nc localhost 8081

CONNECT github.com.com:80 HTTP/1.1
content-length: 0
date: Sat, 22 Sep 2018 05:26:58 GMT


CONNECT github.com:80 HTTP/1.1
content-length: 0
Date: 2018-09-22T21:17:17.971



GET / HTTP/1.0
GET / HTTP/1.1
HTTP2 是文件头里 Connection: Upgrade, HTTP2-Settings

CONNECT www.web-tinker.com:80 HTTP/1.1
Host: www.web-tinker.com:80
Proxy-Connection: Keep-Alive
Proxy-Authorization: Basic *
Content-Length: 0

HTTP/1.1 200 Connection Established


GET / HTTP/1.1
content-length: 0
Host: biluohc.me:8080
Date: 2018-09-22T21:17:17.971


CONNECT biluohc.me:8080 HTTP/1.1
content-length: 0
date: Sat, 22 Sep 2018 05:26:58 GMT