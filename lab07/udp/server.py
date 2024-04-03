import random
import socket
import sys

BUFFER_SIZE = 1024
PACKET_LOSS_THRESHOLD = 20

if __name__ == '__main__':
    port = int(sys.argv[1])
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    server_socket.bind(('', port))

    while True:
        message, address = server_socket.recvfrom(BUFFER_SIZE)
        coinflip = random.randint(1, 100)
        if coinflip > PACKET_LOSS_THRESHOLD:
            message = str(message, 'UTF-8').upper()
            server_socket.sendto(bytes(message, 'UTF-8'), address)
