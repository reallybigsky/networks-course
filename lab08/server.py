import random
import socket
import threading
import utils as UTILS


# Accepts one client and then communicates only with accepted client
class Server(object):

    def __init__(self, port: int, timeout: float):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
        self.socket.bind(('', port))
        self.timeout = timeout
        self.other_addr = None

        self.write_id = 0
        self.read_id = 0

        self.lock = threading.Lock()
        self.buffer = bytearray()

        self.send_attempts = 0
        self.send_real = 0
        self.received = 0

    def stats(self) -> (int, int, int):
        return self.send_attempts, self.send_real, self.received

    # packet loss imitation
    def send(self, data: bytes):
        self.send_attempts += 1
        if random.random() > UTILS.PACKET_LOSS:
            self.send_real += 1
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
                threading.Thread(target=self.__background_read, daemon=True).start()
                return data, addr

    def write(self, data: bytes):
        with self.lock:
            if self.other_addr is None:
                raise Exception('')

            bytes_sent = 0
            while bytes_sent < len(data):
                packet_len = min(UTILS.PACKET_DATA_LEN, len(data) - bytes_sent)
                packet = UTILS.create_packet(self.write_id, data[bytes_sent:bytes_sent + packet_len])

                while True:
                    self.send(packet)
                    try:
                        response, _ = self.socket.recvfrom(UTILS.PACKET_LEN)
                        self.received += 1
                        ack_payload, ok = UTILS.check_packet(self.write_id, response)
                        if ok and ack_payload == UTILS.ACK_MAGIC:
                            break

                    except socket.timeout:
                        continue

                self.write_id = (self.write_id + 1) % 2
                bytes_sent += packet_len

    def read(self, n: int) -> bytes:
        while True:
            with self.lock:
                if len(self.buffer) < n:
                    continue

                res = self.buffer[:n]
                self.buffer = self.buffer[n:]
                return res

    def __background_read(self):
        while True:
            with self.lock:
                try:
                    data, addr = self.socket.recvfrom(UTILS.PACKET_LEN)
                    self.received += 1
                    if addr != self.other_addr:
                        continue

                    payload, ok = UTILS.check_packet(self.read_id, data)
                    if ok:
                        packet = UTILS.create_packet(self.read_id, UTILS.ACK_MAGIC)
                        self.read_id = (self.read_id + 1) % 2
                        self.send(packet)
                        self.buffer += payload
                    else:
                        packet = UTILS.create_packet((self.read_id + 1) % 2, UTILS.ACK_MAGIC)
                        self.send(packet)

                except socket.timeout:
                    continue
