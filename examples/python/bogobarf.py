""" Python implementation of bogobarf ideas.
"""

# from tornado.tcpclient import TCPClient
import asyncio
import struct
import cbor


class Node:
    def __init__(self, name):
        self.name = name
    
    async def connect(self):
        self.reader, self.writer = await asyncio.open_connection(host='127.0.0.1', port=6142)

    async def disconnect(self):
        self.writer.close()
        await self.writer.wait_closed()

    async def write_msg(self, data):
        msg = {
            'id': int(data),
            'text': 'w00000t',
        }
        data = cbor.dumps(msg)
        header = struct.pack('>I', len(data))
        await self.write_bytes(header + data)

    async def write_bytes(self, data):
        self.writer.write(data)
        await self.writer.drain()

    async def read_msg(self):
        header = await self.read_bytes(4)
        assert len(header) == 4
        length, = struct.unpack('>I', header)
        data = await self.read_bytes(length)
        assert len(data) == length
        msg = cbor.loads(data)
        return msg

    async def read_bytes(self, amount):
        return await self.reader.readexactly(4)

    def publisher(self, topic):
        return Publisher(self)


class Publisher:
    def __init__(self, node):
        self.node = node
    
    async def write(self, data):
        print(data)
        await self.node.write_msg(data)