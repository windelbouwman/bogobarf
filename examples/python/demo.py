
# import bogobarf as bb

import tornado
import socket
import struct
import cbor


class Node:
    def __init__(self, name):
        self.connect()
    
    def connect(self):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect(('localhost', 6142))

    def disconnect(self):
        self.socket.close()

    def write_msg(self, data):
        msg = {
            'id': int(data),
            'text': 'w00000t',
        }
        data = cbor.dumps(msg)
        header = struct.pack('>I', len(data))
        self.socket.sendall(header + data)

    def read_msg(self):
        header = self.socket.recv(4)
        assert len(header) == 4
        length, = struct.unpack('>I', header)
        data = self.socket.recv(length)
        assert len(data) == length
        msg = cbor.loads(data)
        return msg

    def publisher(self, topic):
        return Publisher(self)


class Publisher:
    def __init__(self, node):
        self.node = node
    
    def write(self, data):
        print(data)
        self.node.write_msg(data)


node = Node('demo')

# node.write_msg('bogobarf-python!'.encode())

pub = node.publisher('/bla')
# for x in range(10000):
for x in range(10):
    pub.write(x)

print('got back:', node.read_msg())
node.disconnect()
