import threading
from service.server import run as run_server
from service.client import run as run_client

IS_QUIC = False

PORT = 12345
QUIC_PORT = 23456
TCP_DATA_MB = 10

if __name__ == "__main__":
    server_thread = threading.Thread(target=run_server, args=("0.0.0.0", PORT, 4096))
    client_thread = threading.Thread(target=run_client, args=("127.0.0.1", QUIC_PORT if IS_QUIC else PORT, TCP_DATA_MB, 4096))
    server_thread.start()
    client_thread.start()