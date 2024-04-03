import socket
import sys
import datetime

BUFFER_SIZE = 1024
REQUEST_COUNT = 10
TIMEOUT_SECONDS = 1

if __name__ == '__main__':
    message = sys.argv[1]
    server_ip = sys.argv[2]
    server_port = int(sys.argv[3])

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    client_socket.settimeout(TIMEOUT_SECONDS)

    addr = (server_ip, server_port)

    min_rtt = 1000000000
    max_rtt = 0
    sum_rtt = 0
    cnt = 0

    print(f'PING {addr} with message {message}\n')

    for request in range(REQUEST_COUNT):
        start = datetime.datetime.utcnow()
        client_socket.sendto(bytes(message, 'UTF-8'), addr)

        print(f'Ping {request} {start}')

        try:
            response, _ = client_socket.recvfrom(BUFFER_SIZE)
            end = datetime.datetime.utcnow()
            elapsed = end - start
            rtt = elapsed.seconds * 1000 + elapsed.microseconds / 1000

            min_rtt = min(min_rtt, rtt)
            max_rtt = max(max_rtt, rtt)
            sum_rtt += rtt
            cnt += 1

            print(f'\tGot {str(response, "UTF-8")} from {addr} time={rtt} ms')
        except socket.timeout:
            print("\tRequest timed out")

    print(f'--- {addr} statisticts ---')
    print(f'{REQUEST_COUNT} packets transmitted, {cnt} received, {int((REQUEST_COUNT - cnt) / REQUEST_COUNT * 100)}% packet loss, time {sum_rtt}ms')
    print(f'rtt min/avg/max = {min_rtt}/{sum_rtt / cnt}/{max_rtt} ms')
