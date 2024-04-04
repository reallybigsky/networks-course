import struct
import socket
import sys
import time

BUFFER_SIZE = 1024

class Client:
    def __init__(self, addr, last_msg_idx, last_msg_time):
        self.addr = addr
        self.last_msg_idx = last_msg_idx
        self.last_msg_time = last_msg_time


if __name__ == '__main__':
    timeout = int(sys.argv[1])
    port = int(sys.argv[2])
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    server_socket.bind(('', port))

    server_socket.settimeout(timeout)

    clients = dict()

    while True:
        try:
            message, address = server_socket.recvfrom(BUFFER_SIZE)
            msg_idx, msg_time = struct.unpack('Id', message)
            if address not in clients.keys():
                print(f'{time.time()} INFO: New client {address}')
                clients[address] = Client(address, msg_idx, msg_time)

            if clients[address].last_msg_idx + 1 != msg_idx:
                for idx in range(clients[address].last_msg_idx, msg_idx):
                    print(f'{time.time()} INFO: Lost packet #{idx} from {address}')

            clients[address].last_msg_idx = msg_idx
            clients[address].last_msg_time = msg_time

        finally:
            curr_time = time.time()
            for client in clients.values():
                if client.last_msg_time + timeout < curr_time:
                    print(f'{time.time()} INFO: Lost heartbeat from {client.addr}, last msg time: {client.last_msg_time}')
