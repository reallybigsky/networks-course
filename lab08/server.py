import random
import socket
from lab08 import utils as UTILS


# Accepts one client and then communicates only with accepted client
class Server(object):

    def __init__(self, port: int, timeout: float):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
        self.socket.bind(('', port))
        self.timeout = timeout
        self.write_id = 0
        self.read_id = 0
        self.other_addr = None

    # packet loss imitation
    def send(self, data: bytes):
        if random.random() > UTILS.PACKET_LOSS:
            self.socket.sendto(data, self.other_addr)

    def accept(self) -> (str, int):
        if self.other_addr is not None:
            raise Exception('')

        while True:
            data, addr = self.socket.recvfrom(UTILS.PACKET_LEN)
            payload, ok = UTILS.check_packet(0, data)
            if ok and payload == UTILS.HANDSHAKE_REQUEST:
                self.other_addr = addr
                response = UTILS.create_packet(1, UTILS.HANDSHAKE_RESPONSE)
                self.socket.sendto(response, addr)
                self.socket.settimeout(self.timeout)
                return data, addr

    def write(self, data: bytes):
        if self.other_addr is None:
            raise Exception('')

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
                if addr != self.other_addr:
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
