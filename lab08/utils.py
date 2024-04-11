from lab08 import checksum
import struct

PACKET_DATA_LEN = 1024
PACKET_HEADER_LEN = 6
CHECKSUM_LEN = 2
PACKET_LEN = PACKET_DATA_LEN + PACKET_HEADER_LEN

PACKET_LOSS = 0.3

ACK_MAGIC = b'MAGIC_ABOBA_ACK'
HANDSHAKE_REQUEST = b'hello'
HANDSHAKE_RESPONSE = b'world'
WRITE_RESEND_THRESHOLD = 100


def check_packet(expected_id: int, packet: bytes) -> (bytes, bool):
    cs, leftover = struct.unpack(f'H{len(packet) - CHECKSUM_LEN}s', packet)
    if not checksum.check_cs16(cs, leftover):
        return None, False

    actual_id, payload = struct.unpack(f'I{len(leftover) - PACKET_HEADER_LEN + CHECKSUM_LEN}s', leftover)
    if actual_id != expected_id:
        return None, False

    return payload, True


def create_packet(packet_id: int, data: bytes) -> bytes:
    buf = bytearray(PACKET_HEADER_LEN + len(data))
    struct.pack_into(f'I{len(data)}s', buf, CHECKSUM_LEN, packet_id, data)
    cs = checksum.calc_cs16(buf)
    buf[:CHECKSUM_LEN] = cs.tobytes()
    return buf

