# FlightSim Backend (Rust)

FlightSim Backend for SyncSeeker by Rust (public version)

这是SKYline在线连飞服务器连线数据**轻量化版后端**的公开版本，为SyncSeeker V2提供实时在线数据API

开发与测试环境用的是SKYline Dynamic Server (golang edition, latest)，理论上支持目前市面上全部同输出格式的FSD(session与FlightObject格式)

**暂不支持whazzup格式的FSD，whazzup输出和数据分析比较低效且老旧，现已废弃，目前暂无适配计划**

## 快速开始

1. 复制 `.env.example` 为 `.env`
2. 把 `EXTERNAL_API_URL` 改成你 FSD 在线数据接口的地址（http/https格式）
3. 完善 `.env`文件项，RSA公私钥需使用2048 bits长度
4. `cargo run --release`

默认监听地址可在 `.env` 里用 `BIND_ADDR` 修改，例如 `0.0.0.0:8080`。

## 环境变量

| 变量                  | 说明                              | 默认值             |
| --------------------- | --------------------------------- | ------------------ |
| EXTERNAL_API_URL      | FSD 在线数据接口地址              | 必填               |
| RSA_PUBLIC_KEY        | PEM 格式的 RSA 公钥，用于签名验证 | 必填               |
| BIND_ADDR             | 监听地址                          | `127.0.0.1:3000` |
| POLL_INTERVAL_SECONDS | 向上游轮询间隔                    | 15                 |

## 签名验证

所有 `/map/v2/*` 接口要求：

- Header `x-id`: 请求方唯一标识
- Header `x-timestamp`: Unix 时间戳（秒）
- Header `x-signature`: RSA-PSS 对「id + timestamp」的签名（Base64）

## 协议

[GPL-3.0 license](LICENSE) , 使用请遵守开源协议规范。


欢迎积极提issue与PR，喜欢的话也欢迎点个star～
