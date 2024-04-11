import struct
import filecmp
import time

from lab08 import client

if __name__ == '__main__':
    socket = client.Client("127.0.0.1", 8080, 0.05)

    print("Sending Alice...")
    alice = open("alice.txt", "rb")
    alice_data = alice.read()
    alice.close()

    socket.write(len(alice_data).to_bytes(4, byteorder='little'))
    socket.write(alice_data)

    print("Alice sent!")
    time.sleep(5)  # filter ACK from last packet
    print("Collecting TL...")

    tl = open("TL_doc_copy.pdf", "wb")
    size = struct.unpack("I", socket.read())[0]
    curr_size = 0

    while curr_size < size:
        data = socket.read()
        tl.write(data)
        curr_size += len(data)

    tl.close()

    print("Got TL!")
    print("Checking...")

    assert filecmp.cmp("TL_doc.pdf", "TL_doc_copy.pdf")

    print("OK!")
