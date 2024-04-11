import struct
import filecmp
import time

from lab08 import server

if __name__ == '__main__':
    socket = server.Server(8080, 0.05)
    client_ip, client_port = socket.accept()

    print("Collecting Alice...")

    alice = open("alice_copy.txt", "wb")
    size = struct.unpack("I", socket.read())[0]
    curr_size = 0

    while curr_size < size:
        data = socket.read()
        alice.write(data)
        curr_size += len(data)

    alice.close()

    print("Got Alice!")
    time.sleep(5)  # filter ACK from last packet
    print("Sending TL...")

    tl = open("TL_doc.pdf", "rb")
    tl_data = tl.read()
    tl.close()

    socket.write(len(tl_data).to_bytes(4, byteorder='little'))
    socket.write(tl_data)

    print("TL sent!")
    print("Checking...")

    assert filecmp.cmp("alice.txt", "alice_copy.txt")

    print("OK!")
