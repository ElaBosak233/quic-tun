import subprocess
import time
import threading
from service.server import run as run_server
from service.client import run as run_client

import matplotlib.pyplot as plt

# 参数配置
LO_INTERFACE = "lo"
PORT = 12345
QUIC_PORT = 4433
LOSS_RATES = [0, 1, 3, 5, 10]  # 单位：百分比
TCP_DATA_MB = 100

# 工具路径
QUIC_TUN_BIN = "quic-tun"
SERVER_BIND = f"127.0.0.1:{QUIC_PORT}"
SERVER_DEST = f"127.0.0.1:{PORT}"
CLIENT_BIND = f"127.0.0.1:{PORT}"
CLIENT_DEST = f"127.0.0.1:{QUIC_PORT}"

def set_loss(loss_percent):
    print(f"[tc] 设置丢包率为 {loss_percent}%")
    subprocess.run(f"sudo tc qdisc replace dev {LO_INTERFACE} root netem loss {loss_percent}%", shell=True)

def clear_tc():
    subprocess.run(f"sudo tc qdisc del dev {LO_INTERFACE} root", shell=True)

def run_tcp_server():
    thread = threading.Thread(target=run_server, args=(), daemon=True)
    thread.start()

def run_tcp_client():
    return subprocess.run(["python3", "tcp_client.py"], capture_output=True, text=True)

def start_quic_tun_server():
    return subprocess.Popen([QUIC_TUN_BIN, "serve", "--bind", SERVER_BIND, "--dest", SERVER_DEST])

def start_quic_tun_client():
    return subprocess.Popen([QUIC_TUN_BIN, "connect", "--bind", CLIENT_BIND, "--dest", CLIENT_DEST, "--insecure"])

def extract_throughput(output):
    for line in output.splitlines():
        if "Throughput" in line:
            return float(line.split(":")[-1].split()[0])  # MB/s
    return 0.0

def kill_process(proc):
    if proc and proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

def main():
    results = []

    for loss in LOSS_RATES:
        print(f"\n=== 测试丢包率 {loss}% ===")
        set_loss(loss)

        # 启动服务端和 quic-tun
        tcp_server = run_tcp_server()
        time.sleep(1)
        quic_server = start_quic_tun_server()
        time.sleep(1)
        quic_client = start_quic_tun_client()
        time.sleep(1)

        # 启动 TCP 客户端
        tcp_client_result = run_tcp_client()
        throughput = extract_throughput(tcp_client_result.stdout)
        print(f"[✓] 吞吐量: {throughput:.2f} MB/s")
        results.append((loss, throughput))

        # 清理进程
        kill_process(quic_client)
        kill_process(quic_server)
        kill_process(tcp_server)
        clear_tc()
        time.sleep(1)

    # 绘图
    losses = [x[0] for x in results]
    throughputs = [x[1] for x in results]

    plt.plot(losses, throughputs, marker='o')
    plt.xlabel("Packet Loss (%)")
    plt.ylabel("Throughput (MB/s)")
    plt.title("QUIC Tunnel Throughput under Packet Loss")
    plt.grid(True)
    plt.savefig("loss_vs_throughput.png")
    plt.show()

if __name__ == "__main__":
    try:
        main()
    finally:
        clear_tc()
