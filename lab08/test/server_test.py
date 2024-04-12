import filecmp
import os
import struct
import sys

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import server

if __name__ == '__main__':
    socket = server.Server(8080, 0.05)
    client_ip, client_port = socket.accept()

    print("Collecting Alice...")

    alice = open("alice_copy.txt", "wb")
    size = struct.unpack("I", socket.read(4))[0]
    data = socket.read(size)
    alice.write(data)
    alice.close()

    print("Got Alice!")
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

    send_attempts, send_real, received = socket.stats()
    print("STATS:")
    print("\tAttempts to send: ", send_attempts)
    print("\tPackets sent: ", send_real)
    print("\tPackets received: ", received)
