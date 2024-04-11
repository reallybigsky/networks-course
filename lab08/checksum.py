import numpy as np


def calc_cs16(data: bytes) -> np.uint16:
    cs = 0
    a = np.frombuffer(data, dtype=np.uint16) if len(data) % 2 == 0 else np.frombuffer(data + b'\x00', dtype=np.uint16)
    for s in a:
        cs += s

    return np.invert(np.uint16(cs))


def check_cs16(cs: np.uint16, data: bytes) -> bool:
    tmp = 0
    a = np.frombuffer(data, dtype=np.uint16) if len(data) % 2 == 0 else np.frombuffer(data + b'\x00', dtype=np.uint16)
    for s in a:
        tmp += s

    return np.invert(np.uint16(tmp)) == cs
