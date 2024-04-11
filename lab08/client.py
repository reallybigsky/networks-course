import random
import socket
from lab08 import utils as UTILS


class Client(object):

    def __init__(self, server_addr: str, server_port: int, timeout: float):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
        self.socket.settimeout(timeout)
        self.addr = (server_addr, server_port)
        self.write_id = 0
        self.read_id = 0

        request = UTILS.create_packet(0, UTILS.HANDSHAKE_REQUEST)
        self.socket.sendto(request, self.addr)
        response, addr = self.socket.recvfrom(UTILS.PACKET_LEN)
        if addr != self.addr:
            raise Exception('')

        response_payload, ok = UTILS.check_packet(1, response)
        if not ok or response_payload != UTILS.HANDSHAKE_RESPONSE:
            raise Exception('')

    # packet loss imitation
    def send(self, data: bytes):
        if random.random() > UTILS.PACKET_LOSS:
            self.socket.sendto(data, self.addr)

    def write(self, data: bytes):
        bytes_sent = 0
        while bytes_sent < len(data):
            packet_len = min(UTILS.PACKET_DATA_LEN, len(data) - bytes_sent)
            packet = UTILS.create_packet(self.write_id, data[bytes_sent:bytes_sent + packet_len])

            # there is a condition when last ACK is lost and other
            # side does not read packets from us and does not write
            # ACK for last packet. So we can block here infinitely
            # waiting for ACK, which won't be sent anymore
            for cnt in range(UTILS.WRITE_RESEND_THRESHOLD):
                self.send(packet)
                try:
                    response, _ = self.socket.recvfrom(UTILS.PACKET_LEN)
                    ack_payload, ok = UTILS.check_packet(self.write_id, response)
                    if ok and ack_payload == UTILS.ACK_MAGIC:
                        break

                except socket.timeout:
                    continue

            self.write_id = (self.write_id + 1) % 2
            bytes_sent += packet_len

    def read(self) -> bytes:
        while True:
            try:
                data, addr = self.socket.recvfrom(UTILS.PACKET_LEN)
                if addr != self.addr:
                    continue

                payload, ok = UTILS.check_packet(self.read_id, data)
                if ok:
                    packet = UTILS.create_packet(self.read_id, UTILS.ACK_MAGIC)
                    self.read_id = (self.read_id + 1) % 2
                    self.send(packet)
                    return payload
                else:
                    packet = UTILS.create_packet((self.read_id + 1) % 2, UTILS.ACK_MAGIC)
                    self.send(packet)

            except socket.timeout:
                continue

