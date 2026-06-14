import http.server
import socketserver
import socket
import os
import sys

PORT = int(os.environ.get('DEV_PORT', 1422))
DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'src')

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def end_headers(self):
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        self.send_header('Pragma', 'no-cache')
        self.send_header('Expires', '0')
        super().end_headers()

    def do_GET(self):
        path = self.path.split('?')[0]
        if path == '/favicon.ico':
            self.send_response(204)
            self.end_headers()
            return
        if path in ('/overlay', '/overlay.html'):
            self.path = '/overlay.html'
        elif path in ('/confetti', '/confetti.html'):
            self.path = '/confetti.html'
        elif path in ('/', '/main', '/main.html'):
            self.path = '/main.html'
        elif path in ('/quick-add', '/quick-add.html'):
            self.path = '/quick-add.html'
        elif path in ('/celebration', '/celebration.html'):
            self.path = '/celebration.html'
        elif path.startswith('/css/') or path.startswith('/js/') or path.startswith('/assets/'):
            pass
        else:
            self.send_response(404)
            self.end_headers()
            return
        return super().do_GET()

if __name__ == '__main__':
    os.chdir(DIR)
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(('', PORT), Handler) as httpd:
        print(f'Serving frontend at http://localhost:{PORT}')
        httpd.serve_forever()
