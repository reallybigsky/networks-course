import struct
import socket
import sys
import time
import random

PACKET_LOSS_THRESHOLD = 20

if __name__ == '__main__':
    idx = 0
    heartbeat_period = int(sys.argv[1])
    server_ip = sys.argv[2]
    server_port = int(sys.argv[3])

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    addr = (server_ip, server_port)

    while True:
        curr_time = time.time()
        coinflip = random.randint(1, 100)
        if coinflip > PACKET_LOSS_THRESHOLD:
            client_socket.sendto(struct.pack('Id', idx, curr_time), addr)
        idx += 1
        time.sleep(heartbeat_period)
