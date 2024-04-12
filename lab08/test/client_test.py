import struct
import filecmp
import sys
import os

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import client

if __name__ == '__main__':
    socket = client.Client("127.0.0.1", 8080, 0.05)

    print("Sending Alice...")
    alice = open("alice.txt", "rb")
    alice_data = alice.read()
    alice.close()

    socket.write(len(alice_data).to_bytes(4, byteorder='little'))
    socket.write(alice_data)

    print("Alice sent!")
    print("Collecting TL...")

    tl = open("TL_doc_copy.pdf", "wb")
    size = struct.unpack("I", socket.read(4))[0]
    data = socket.read(size)
    tl.write(data)
    tl.close()

    print("Got TL!")
    print("Checking...")

    assert filecmp.cmp("TL_doc.pdf", "TL_doc_copy.pdf")

    print("OK!")

    send_attempts, send_real, received = socket.stats()
    print("STATS:")
    print("\tAttempts to send: ", send_attempts)
    print("\tPackets sent: ", send_real)
    print("\tPackets received: ", received)
