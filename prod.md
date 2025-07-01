Smokeping-rs 产品需求文档 (PRD)
版本: 1.2
日期: 2025年6月30日
1. 概述
1.1. 项目愿景
smokeping-rs 旨在成为一个现代化、高性能、易于部署和扩展的网络延迟监控解决方案。它将借鉴经典工具 Smokeping 的核心思想，利用 Rust 的高并发和内存安全特性实现一个强大的数据采集后端，并结合 Vue.js 构建一个功能丰富、交互流畅的现代化 Web 前端，为网络工程师、系统管理员和开发者提供直观、实时的网络质量可视化分析。
1.2. 目标用户
网络工程师: 需要监控关键网络链路的延迟和丢包率。
系统/运维管理员: 需要确保服务器和服务的网络可达性和响应质量。
云服务用户: 需要监控云提供商的网络性能。
开发者: 需要在开发和测试阶段分析应用的网络依赖性。
1.3. 核心价值
高性能: 后端采用 Rust 编写，利用异步 IO 和多线程能力，可以同时对大量目标进行高频次探测，资源占用低。
现代化 UI: 前端采用 Vue.js 和现代图表库，提供比传统 Smokeping 更美观、更具交互性的数据可视化体验。
极简部署: 前端资源被直接嵌入后端二进制文件，整个应用（除数据库外）是一个独立的、自包含的服务，通过 docker-compose 可实现一键部署。
可扩展性: 清晰的架构设计，便于未来增加新的探测类型（如 HTTP、DNS 等）和告警通知方式。
2. 功能需求
2.1. 后端核心功能
多目标探测 (Probing):
支持通过配置文件或 API 动态添加、删除、修改监控目标。
每个目标可独立配置探测频率、探测包大小等参数。
多协议支持:
ICMP (Ping): 核心探测方式，用于测量网络延迟和丢包率。
(未来扩展) TCP Ping: 向指定端口发送 TCP SYN 包来测量握手延迟。
(未来扩展) HTTP(S) Get: 测量获取一个 HTTP(S) 端点的响应时间。
数据采集与存储:
精确记录每次探测的往返时间 (RTT)。
统计在每个时间窗口内的丢包数量。
将采集到的数据持久化到数据库。
Web 服务 (一体化):
提供一套 RESTful API，供前端调用。
提供 WebSocket 服务，用于向前端实时推送最新的探测数据。
直接提供前端静态资源服务 (HTML, JS, CSS)。
2.2. 前端功能需求
仪表盘 (Dashboard):
集中展示所有监控目标的概览，包括当前状态（正常、延迟高、丢包）、实时延迟和丢包率。
目标列表应支持搜索和分组。
详细视图 (Detailed View):
点击任一目标可进入详细视图。
使用图表（类似 Smokeping 的烟雾图）展示选定时间范围内的延迟分布。
在图表下方叠加一个丢包率图。
提供时间范围选择器。
实时更新:
图表应能通过 WebSocket 接收实时数据并动态更新。
目标管理:
提供 UI 界面来添加、编辑和删除监控目标。
3. 技术架构
3.1. 整体架构图
+-----------------------------------------------------------------------------------+
| 用户浏览器 (Vue.js Frontend)                                                       |
| +----------------------+  +-------------------------+  +-------------------------+ |
| |   Dashboard          |  |   Detailed View (Charts)|  |   Target Management     | |
| +----------------------+  +-------------------------+  +-------------------------+ |
|        |                                                                          |
|        +---------------------> HTTP/S & WebSocket <---------------------+        |
|                                                                         |         |
+-------------------------------------------------------------------------+---------+
                                      |
                                      v
+-------------------------------------+---------------------------------------------+
|                                                                                   |
| Rust Backend (Axum) - 单一容器                                                    |
| +---------------------+  +--------------------+  +------------------+  +----------+ |
| | Static File Server  |  | REST API Endpoint  |  | WebSocket Handler|  | Probing  | |
| | (axum-embed)        |  | (GET/POST targets) |  | (Real-time Push) |  | Engine   | |
| +---------------------+  +--------------------+  +------------------+  +----------+ |
|             |                     |                     |                     |     |
|             +---------------------+---------------------+---------------------+     |
|                                   v                                                 |
| +---------------------------------------------------------------------------------+ |
| |   ORM (SeaORM) & DB Connectors (influxdb2)                                      | |
| +---------------------------------------------------------------------------------+ |
|             |                                     |                               |
+-------------+-------------------------------------+-------------------------------+
              |                                     |
              v                                     v
+-------------+------------------+   +--------------+--------------------------------+
| Relational Database (SQLite)   |   | Time-Series Database (InfluxDB)              |
| (via volume mount)             |   | (separate container)                         |
| +----------------------------+ |   | +------------------------------------------+ |
| | Table: targets             | |   | | Measurement: probe_data                  | |
| +----------------------------+ |   | +------------------------------------------+ |
+--------------------------------+   +----------------------------------------------+



3.2. 后端 (Rust)
Web 框架: Axum。
静态文件嵌入: axum-embed。用于将编译好的 Vue 前端静态文件直接嵌入到最终的 Rust 二进制文件中。
异步运行时: Tokio。
ICMP 探测库: 使用 surge-ping。
数据库交互:
ORM: SeaORM，用于操作 SQLite。
时序数据库客户端: influxdb2 crate，用于与 InfluxDB 交互。
配置管理: config crate。
3.3. 前端 (Vue.js)
构建工具: Vite。
核心框架: Vue 3 (Composition API)。
UI 组件库: Element Plus。
图表库: Apache ECharts。
状态管理: Pinia。
API 请求: Axios。
3.4. 交互协议
所有交互（API、WebSocket、静态文件）均由 Rust 后端在同一个端口上提供服务。
REST API (HTTP/S): 路由前缀为 /api。
GET /api/targets
POST /api/targets
GET /api/targets/:id/data?start_time=<ts>&end_time=<ts>
WebSocket: 路由为 /ws。
前端资源: 所有其他路由 (/, /assets/*, etc.) 都将由 axum-embed 提供服务。
4. 数据库设计
4.1. 数据库选型
4.1.1. 配置数据存储: SQLite
理由: 轻量级、无服务器、文件即数据库。非常适合存储应用配置、监控目标列表等不频繁写入的数据。
4.1.2. 探测数据存储: InfluxDB
理由: 专门为时间序列数据设计的高性能数据库。具备极高的写入吞吐量和快速的时间范围查询能力。
4.2. 数据模型 (Data Models)
targets (SQLite 表)
id (INTEGER PRIMARY KEY AUTOINCREMENT)
name (TEXT NOT NULL)
host (TEXT NOT NULL)
probe_type (TEXT DEFAULT 'icmp')
probe_interval_secs (INTEGER DEFAULT 60)
is_active (INTEGER DEFAULT 1)
created_at (TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)
probe_data (InfluxDB Measurement)
Measurement: probe_data
Tags: target_id (String), is_lost (Boolean)
Fields: rtt_ms (Float)
Timestamp: time
5. 部署方案
5.1. Docker 容器化
项目将由两个主要的服务容器组成：backend 和 influxdb。前端不再需要独立的容器。
后端 Dockerfile (backend/Dockerfile)
强烈推荐多阶段构建 (multi-stage build) 来保持镜像的苗条。
第一阶段 (Frontend Build): 使用 node:lts-alpine 镜像，复制前端代码，运行 npm install 和 npm run build，生成静态文件到 dist 目录。
第二阶段 (Backend Build & Embed): 使用 rust:latest 镜像，将第一阶段生成的 dist 目录复制到后端项目的一个子目录（例如 frontend/dist），然后编译 Rust 项目。axum-embed 会在编译时将这些文件包含进来。
第三阶段 (Final Image): 使用一个轻量级的镜像（如 gcr.io/distroless/cc-debian11 或 alpine），仅将第二阶段编译好的单一二进制文件复制进去。
5.2. Docker Compose (docker-compose.yml)
docker-compose 的配置被简化，现在只包含两个服务。
version: '3.8'

services:
  influxdb:
    image: influxdb:2.7
    container_name: smokeping_influxdb
    environment:
      - DOCKER_INFLUXDB_INIT_MODE=setup
      - DOCKER_INFLUXDB_INIT_USERNAME=admin
      - DOCKER_INFLUXDB_INIT_PASSWORD=password
      - DOCKER_INFLUXDB_INIT_ORG=smokeping-org
      - DOCKER_INFLUXDB_INIT_BUCKET=smokeping
      - DOCKER_INFLUXDB_INIT_ADMIN_TOKEN=YourSecureTokenHere
    volumes:
      - influxdb_data:/var/lib/influxdb2
    ports:
      - "8086:8086"
    restart: unless-stopped

  # 前端容器已被移除
  # backend 容器现在是唯一的应用入口
  backend:
    build:
      context: . # 假设 Dockerfile 在根目录，可以协调前后端构建
    container_name: smokeping_app
    depends_on:
      - influxdb
    environment:
      # InfluxDB Connection
      - INFLUXDB_URL=http://influxdb:8086
      - INFLUXDB_TOKEN=YourSecureTokenHere
      - INFLUXDB_ORG=smokeping-org
      - INFLUXDB_BUCKET=smokeping
      # SQLite Connection
      - DATABASE_URL=/data/smokeping.db
      # Web Server Port
      - ROCKET_PORT=8080
    volumes:
      - sqlite_data:/data # 挂载一个卷来持久化 SQLite 数据库文件
    ports:
      - "8080:8080" # 将应用端口暴露给主机
    restart: unless-stopped

volumes:
  influxdb_data:
  sqlite_data:


6. 里程碑 (Milestones)
M1: 核心后端开发 (2周)
搭建 Rust 项目框架 (Axum, SeaORM, InfluxDB client)。
实现 ICMP 探测引擎。
完成数据库模型设计并在后端集成。
实现基础的 targets 管理 API。
M2: 核心前端与集成构建 (3周)
搭建 Vue 项目框架。
实现 Dashboard 和详细视图页面。
配置 axum-embed 并设置好前后端联合构建流程。
集成 WebSocket，实现图表实时更新。
M3: 容器化与集成 (1周)
编写统一的 Dockerfile 和 docker-compose.yml。
完成所有服务的容器化部署和联调。
M4: 功能完善与测试 (2周)
完善前端的目标管理 UI。
利用 InfluxDB 的特性实现数据聚合/下采样。
编写单元测试和集成测试。
发布 v1.0.0。
