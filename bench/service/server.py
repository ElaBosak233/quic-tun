import socket
import time
from util.logger import log

def run(bind_ip="0.0.0.0", port=12345, buffer_size=4096):
    total_bytes = 0

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as server_sock:
        server_sock.bind((bind_ip, port))
        server_sock.listen(1)
        log.info(f"[+] Listening on {bind_ip}:{port}")
        conn, addr = server_sock.accept()
        log.info(f"[+] Connection from {addr}")

        with conn:
            start_time = time.time()
            while True:
                data = conn.recv(buffer_size)
                if not data:
                    break
                total_bytes += len(data)

    end_time = time.time()
    duration = end_time - start_time
    mb = total_bytes / (1024 * 1024)
    log.info(f"[+] Received {mb:.2f} MB in {duration:.2f} s")
    log.info(f"[+] Throughput: {mb / duration:.2f} MB/s")

if __name__ == "__main__":
    run()