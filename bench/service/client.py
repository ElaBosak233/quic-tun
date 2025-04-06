import socket
import time
from util.logger import log

def run(server_ip="127.0.0.1", port=23456, total_mb=100, buffer_size=4096):
    total_bytes = total_mb * 1024 * 1024
    sent_bytes = 0
    data = b"x" * buffer_size

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((server_ip, port))
        log.info(f"[+] Connected to {server_ip}:{port}")
        start_time = time.time()

        while sent_bytes < total_bytes:
            sent = sock.send(data)
            sent_bytes += sent
            log.debug(f"[+] Sent {sent} bytes, total sent {sent_bytes} bytes")

    end_time = time.time()
    duration = end_time - start_time
    log.info(f"[+] Sent {total_mb:.2f} MB in {duration:.2f} s")
    log.info(f"[+] Throughput: {total_mb / duration:.2f} MB/s")

if __name__ == "__main__":
    run()