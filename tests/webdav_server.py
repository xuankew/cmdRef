#!/usr/bin/env python3
"""cmdref sync 集成测试用的迷你 WebDAV 服务器（仅标准库）。

只实现 cmdref sync 用到的子集：GET / PUT / MKCOL / PROPFIND + Basic 认证。
数据存内存，进程退出即销毁，用于测试登录、推送、拉取与 3-way 合并全链路。

用法: python3 webdav_server.py <port>    # port 传 0 表示随机端口
启动成功后向 stdout 打印 "PORT=<实际端口>"。
"""
import base64
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

USERNAME = "testuser"
PASSWORD = "testpass"

DIRS = {"/"}
FILES = {}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        pass

    # ---------- helpers ----------

    def check_auth(self):
        expected = "Basic " + base64.b64encode(
            f"{USERNAME}:{PASSWORD}".encode()
        ).decode()
        if self.headers.get("Authorization", "") == expected:
            return True
        # 未读完请求体前不能复用连接，直接关闭
        self.close_connection = True
        self.send_response(401)
        self.send_header("WWW-Authenticate", 'Basic realm="cmdref-test"')
        self.send_header("Content-Length", "0")
        self.end_headers()
        return False

    def parent(self, path):
        p = path.rstrip("/")
        return p.rsplit("/", 1)[0] or "/"

    def drain_body(self):
        length = int(self.headers.get("Content-Length") or 0)
        if length:
            self.rfile.read(length)

    def reply(self, status, body=b"", content_type="application/octet-stream"):
        self.send_response(status)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Content-Type", content_type)
        self.end_headers()
        if body:
            self.wfile.write(body)

    # ---------- methods ----------

    def do_GET(self):
        if not self.check_auth():
            return
        body = FILES.get(self.path)
        if body is None:
            self.reply(404)
        else:
            self.reply(200, body)

    def do_PUT(self):
        if not self.check_auth():
            return
        body = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        if self.parent(self.path) not in DIRS:
            self.reply(409)
            return
        FILES[self.path] = body
        self.reply(201)

    def do_MKCOL(self):
        if not self.check_auth():
            return
        self.drain_body()
        path = self.path.rstrip("/") or "/"
        if path in DIRS:
            self.reply(405)
            return
        if self.parent(path) not in DIRS:
            self.reply(409)
            return
        DIRS.add(path)
        self.reply(201)

    def do_PROPFIND(self):
        if not self.check_auth():
            return
        self.drain_body()
        path = self.path.rstrip("/") or "/"
        if path not in DIRS:
            self.reply(404)
            return
        xml = (
            '<?xml version="1.0"?>'
            '<D:multistatus xmlns:D="DAV:">'
            f"<D:response><D:href>{path}</D:href></D:response>"
            "</D:multistatus>"
        ).encode()
        self.reply(207, xml, "application/xml")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"PORT={server.server_address[1]}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
