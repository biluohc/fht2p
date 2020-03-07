# fht2p
[![Actions Status](https://github.com/biluohc/fht2p/workflows/CI/badge.svg)](https://github.com/biluohc/fht2p/actions)

[English](https://github.com/biluohc/fht2p/blob/master/readme.md)

fht2p 是使用 Rust 开发的，跨平台的 HTTP 静态文件服务器，CI测试覆盖了 Linux, MacOS 和 Windows 三大平台。

## 特点
- 可靠： 使用 Rust 实现，集成测试，安全可靠
- 通用： 完全跨平台，Unix-like 和 Windows 系统均可用
- 方便： 静态链接，不依赖 openssl 等外部动态库，下载即可用
- 极速： 异步多线程，基于 tokio 和 hyper 打造，极速响应，海量并发
- 功能： 文件下载上传，目录浏览，断点续传，代理功能，配置选项，应有尽有

### 功能
1. 多路径分享
1. 文件断点续传
1. 可关闭的目录浏览（以及排序等功能）
1. HTTP 缓存
1. 文件上传，目录新建
1. HTTPS（tokio-rustls, 不依赖外部动态库）
1. HTTP 代理（隧道代理, 普通代理)
1. 基本认证（Basic Authentication）
1. 跨域资源共享（CORS）
1. 目录页面压缩（GZIP)
1. 命令行参数
1. 配置文件(格式是json5，类似json但支持注释等)
1. 终端日志可选级别
1. 启动时输出服务的网址，可选输出二维码

### 截图

![snapshot.png](https://raw.githubusercontent.com/biluohc/fht2p/master/config/assets/snapshot.png)

### 安装

#### 下载自 [Releases](https://github.com/biluohc/fht2p/releases)

#### 从源码编译
```sh
    cargo install --git https://github.com/biluohc/fht2p fht2p -f

    fht2p -h
```
##### 或者
```sh
    git clone https://github.com/biluohc/fht2p
    # cargo  install --path fht2p/ fht2p -f

    cd fht2p
    cargo build --release

    ./target/release/fht2p --help
```

### 提示
1. --help 可以查看帮助信息
1. --config-print 可以查看默认配置内容
1. 项目下的config目录里有完整的配置文件示例

2. 关于选项和配置文件的优先级

    默认配置文件位于 `$HOME/.config/fht2p/fht2p.json`, 如果没有，可以新建

    选项总体可以分为四种:
- 第一种是 --help， --version 和 --config-print 程序会很快退出不需要考虑优先级
- 第二种是 --verbose 和 --qr-code 这种优先级忽略的，和其它选项无冲突
- 第三种是 --config 指定配置文件，就会忽略第四种选项
- 第四种是 其他选项和参数，一旦有了就会忽略默认的配置文件（之所以这样是为了防止优先级太复杂）

5. 关于安全和 HTTPS

- HTTP是基于TCP的明文协议，完全没有安全可言，如果需要安全性，一定要使用 HTTPS
- 为了安全，程序默认监听的是本机回环地址（`127.0.0.1`）, 本机外要访问可监听 `0.0.0.0`或 特定的地址并配置防火墙
- 程序默认监听当前目录，请不要分享家目录或者根目录等到网络上，除非你明白在干什么
