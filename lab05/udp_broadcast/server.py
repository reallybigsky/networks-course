import socket
import sys
import datetime
from time import sleep

if __name__ == '__main__':
    port = int(sys.argv[1])

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)

    while True:
        msg = datetime.datetime.utcnow()
        sock.sendto(str.encode(msg.isoformat("T")), ("255.255.255.255", port))
        sleep(1)
