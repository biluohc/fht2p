## 生成私钥
openssl  genrsa -out rsa_private_key.pem 2048  

## 生成公匙
openssl rsa -in rsa_private_key.pem -pubout -out rsa_public_key.pem 

## 创建证书请求
openssl req -new -out cert.csr -key rsa_private_key.pem 

```sh
Country Name (2 letter code) [AU]:CN
State or Province Name (full name) [Some-State]:Hangzhou
Locality Name (eg, city) []:Zhejiang Provice
Organization Name (eg, company) [Internet Widgits Pty Ltd]:fht2p
Organizational Unit Name (eg, section) []:fht2p
Common Name (e.g. server FQDN or YOUR name) []:fht2p
Email Address []:biluohc@qq.com

Please enter the following 'extra' attributes
to be sent with your certificate request
A challenge password []:8J+RnfCfkZ/wn5Gr8J+Q
An optional company name []:fht2p
```

## 自签署根证书，就是不通过CA认证中心自行进行证书签名，这里用是x509。 可选 pem 或 der 格式输出
openssl x509 -req -in cert.csr -out rsa_public_key.pem -outform pem -signkey rsa_private_key.pem -days 3650



