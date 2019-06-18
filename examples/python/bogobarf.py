""" Python implementation of bogobarf ideas.

Message types:
- call: call server function
- ret: response from server
- pub: publish on a topic
- bye: indicate connection termination.

Server API:
- register: set node name.
- list: get topics
- announce: anounce a topic.
- subscribe: enroll for topic publications.
"""

import asyncio
import argparse
import logging
import struct
import cbor


class Node:
    logger = logging.getLogger('bogobarf-client-node')

    """ Client node. """
    def __init__(self, name):
        self.name = name
        self.seq_id = 0
        self._response_queues = {}
        self._pub_handlers = {}  # callbacks to subscriptions
        self._services = {}
    
    async def connect(self):
        self.logger.info('Connecting')
        reader, writer = await asyncio.open_connection(host='127.0.0.1', port=6142)
        self.connection = Connection(reader, writer)
        self._recv_task = asyncio.create_task(self.receiver())
        await self.call_method('register', [self.name])

    async def receiver(self):
        """ Receiver task which gets messages and deals with them. """
        while True:
            msg = await self.connection.read_msg()
            self.logger.debug('Processing msg %s', msg)

            # Determine message type, and then what to do
            # with the message.
            if msg['type'] == 'ret':
                seq = msg['seq']
                queue = self._response_queues[seq]
                await queue.put(msg)
            elif msg['type'] == 'pub':
                topic = msg['topic']
                value = msg['value']
                if topic in self._pub_handlers:
                    callback = self._pub_handlers[topic]
                    await callback(value)
                else:
                    self.logger.error('Unsubscribed msg: %s', msg)
            elif msg['type'] == 'call':  # Service invokation
                # Handle invokation.
                method_name = msg['method']
                args = msg['args']
                seq = msg['seq']
                self.logger.debug('Service %s invoked', msg)
                method = self._services[method_name]
                result = await method(*args)
                # TODO: error handling..
                resp = {
                    'type': 'ret',
                    'seq': msg['seq'],
                    'result': result,
                }
                await self.connection.write_msg(resp)
                # TODO: use write queue as well!
                # out_queue.put(resp)
            else:
                self.logger.error('Unknown msg: %s', msg)

    async def disconnect(self):
        self.logger.info('Disconnecting')
        self._recv_task.cancel()
        msg = {'type': 'bye'}
        await self.connection.write_msg(msg)
        await self.connection.writer.drain()
        self.connection.writer.close()
        await self.connection.writer.wait_closed()
        self.logger.debug('Disconnected')

    # Pub/sub functions
    async def publisher(self, topic):
        await self.call_method('announce', [topic])
        return Publisher(self, topic)

    async def publish_value(self, topic, value):
        """ Publish a value to a topic. """
        self.logger.debug('Publishing value %s to %s', value, topic)
        msg = {'type': 'pub', 'topic': topic, 'value': value}
        await self.connection.write_msg(msg)

    async def subscribe(self, topic, callback):
        self._pub_handlers[topic] = callback
        await self.call_method('subscribe', [topic])

    async def unsubscribe(self, topic):
        await self.call_method('unsubscribe', [topic])

    # Service registrations:
    async def register_service(self, service, handler):
        self._services[service] = handler
        await self.call_method('provide', [service])

    # Remote procedure call (RPC) functions
    async def call_method(self, name, args):
        self.logger.debug('Calling method %s', name)
        seq = self._new_seq()
        req = {'type': 'call', 'seq': seq, 'method': name, 'args': args}
        await self.connection.write_msg(req)

        # Fetch proper response:
        assert seq not in self._response_queues
        reply_queue = asyncio.Queue()
        self._response_queues[seq] = reply_queue
        resp = await reply_queue.get()
        self.logger.debug(f'checking response {resp} for sequence number {seq}')
        assert resp['seq'] == seq
        return resp['result']

    def _new_seq(self):
        seq = self.seq_id
        self.seq_id += 1
        return seq


class Publisher:
    def __init__(self, node, topic):
        self.node = node
        self.topic = topic
    
    async def write(self, value):
        await self.node.publish_value(self.topic, value)


class Connection:
    """ Message passing connection. """
    logger = logging.getLogger('bogobarf-connection')

    def __init__(self, reader, writer):
        self.reader = reader
        self.writer = writer

    async def write_msg(self, msg):
        data = cbor.dumps(msg)
        self.logger.debug(f'Packing {len(data)} bytes')
        header = struct.pack('>I', len(data))
        await self.write_bytes(header + data)

    async def write_bytes(self, data):
        self.writer.write(data)
        await self.writer.drain()

    async def read_msg(self):
        header = await self.read_bytes(4)
        assert len(header) == 4
        length, = struct.unpack('>I', header)
        self.logger.debug(f'Reading {length} bytes')
        data = await self.read_bytes(length)
        self.logger.debug(f'Read {len(data)} bytes')
        assert len(data) == length
        msg = cbor.loads(data)
        return msg

    async def read_bytes(self, amount):
        return await self.reader.readexactly(amount)


class Server:
    """ Bogobarf central server. """
    logger = logging.getLogger('bogobarf-server')

    def __init__(self):
        self.peers = []
        self.topics = {}

    async def serve(self):
        self.logger.info('Booting server.')
        server = await asyncio.start_server(self._on_client, host='localhost', port=6142)
        addr = server.sockets[0].getsockname()
        self.logger.info(f'Accepting connections on {addr}')
        async with server:
            await server.serve_forever()

    async def _on_client(self, reader, writer):
        self.logger.info('New client!')
        connection = Connection(reader, writer)
        peer = Peer(self, connection)
        self.peers.append(peer)
        await peer.run()
        self.peers.remove(peer)
        await writer.drain()
        writer.close()

    async def broadcast(self, topic, value):
        """ Broadcast a topic. """
        self.topics[topic] = value
        msg = {
            'type': 'pub',
            'topic': topic,
            'value': value,
        }
        for peer in self.peers:
            if topic in peer.subscriptions:
                await peer.connection.write_msg(msg)


class Peer:
    logger = logging.getLogger('peer')

    def __init__(self, server, connection):
        self.server = server
        self.connection = connection
        self.subscriptions = []  # TODO: this can be a set?
        self._running = False
        self.name = '<unnamed>'

    async def handle_topic_list(self):
        return self.server.topics

    async def handle_register(self, name):
        self.logger.debug(f'Peer registered under name {name}')
        self.name = name

    async def handle_announce(self, topic):
        self.logger.debug(f'Peer announced {topic}')

    async def handle_subscribe(self, topic):
        self.logger.debug(f'Peer subscribed to {topic}')
        self.subscriptions.append(topic)

    async def handle_unsubscribe(self, topic):
        self.logger.debug(f'Peer unsubscribed from {topic}')
        self.subscriptions.remove(topic)

    async def run(self):
        self._running = True
        while self._running:
            msg = await self.connection.read_msg()
            await self._handle_msg(msg)

    async def _handle_msg(self, msg):
        # print(f'peer incoming {msg}')
        if 'type' in msg:
            msg_type = msg['type']
            if msg_type == 'call':
                method_name = msg['method']
                self.logger.debug(f'Function {method_name} called')
                args = msg['args']
                method = getattr(self, f'handle_{method_name}')
                result = await method(*args)
                # TODO: handle exception!
                resp = {
                    'type': 'ret',
                    'seq': msg['seq'],
                    'result': result,
                }
                await self.connection.write_msg(resp)
            elif msg_type == 'pub':
                topic = msg['topic']
                value = msg['value']
                # print('pub', msg)
                await self.server.broadcast(topic, value)
            elif msg_type == 'bye':
                self.logger.debug('Bye peer')
                # termination sentinel
                self._running = False
            else:
                self.logger.error(f'Unknown message type {msg_type}')
        else:
            self.logger.error(f'Unknown message {msg}')


FANCY_HEADER = r"""
   ____     U  ___ u   ____    U  ___ u   ____      _       ____     _____  
U | __")u    \/"_ \/U /"___|u   \/"_ \/U | __")uU  /"\  uU |  _"\ u |" ___| 
 \|  _ \/    | | | |\| |  _ /   | | | | \|  _ \/ \/ _ \/  \| |_) |/U| |_  u 
  | |_) |.-,_| |_| | | |_| |.-,_| |_| |  | |_) | / ___ \   |  _ <  \|  _|/  
  |____/  \_)-\___/   \____| \_)-\___/   |____/ /_/   \_\  |_| \_\  |_|     
 _|| \\_       \\     _)(|_       \\    _|| \\_  \\    >>  //   \\_ )(\\,-  
(__) (__)     (__)   (__)__)     (__)  (__) (__)(__)  (__)(__)  (__|__)(_/ 

"""


def server():
    """ Setup server. """
    logging.info('Booting the bogobarf server.')
    print(FANCY_HEADER)
    server = Server()
    asyncio.run(server.serve())


def client():
    async def main():
        node = Node('demo')
        await node.connect()

        # node.write_msg('bogobarf-python!'.encode())

        pub = await node.publisher('/bla')
        # for x in range(10000):
        for x in range(10):
            msg = {
                'foo': x,
                'blaa': [1,2,3]
            }
            await pub.write(msg)
        await node.disconnect()

    asyncio.run(main())


def topic_list():
    async def main():
        node = Node('demo')
        await node.connect()
        topics = await node.call_method('topic_list', [])
        for topic in topics:
            print(topic)
        await node.disconnect()
    asyncio.run(main())


def topic_pub(topic, value):
    async def main():
        node = Node('demo')
        await node.connect()
        await node.publish_value(topic, value)
        await node.disconnect()
    asyncio.run(main())


def topic_echo(topic):
    async def on_bla(value):
        print(f'got {value} on {topic}')

    async def main():
        node = Node('demo')
        await node.connect()
        await node.subscribe(topic, on_bla)
        while True:
            await asyncio.sleep(1)
        await node.disconnect()
    asyncio.run(main())


def draw_table(rows):
    """ Draw an ascii table. """
    num_columns = [len(row) for row in rows]
    assert num_columns[1:] == num_columns[:-1]
    num_columns = num_columns[0]
    raise NotImplementedError()
    # column_widths = [
    # ]
    # for row in data:
    #     row[c]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-v', help='Increase verbosity', action='count', default=0)
    subparsers = parser.add_subparsers(dest='command', required=True)
    subparsers.add_parser('serve')
    subparsers.add_parser('client')
    pub_parser = subparsers.add_parser('pub')
    pub_parser.add_argument('topic', help='The topic to publish to')
    pub_parser.add_argument('value')
    echo_parser = subparsers.add_parser('echo')
    echo_parser.add_argument('topic', help='The topic to echo')
    subparsers.add_parser('list')
    args = parser.parse_args()

    level = logging.DEBUG if args.v else logging.INFO
    logging.basicConfig(level=level)

    if args.command == 'serve':
        server()
    elif args.command == 'client':
        client()
    elif args.command == 'pub':
        topic_pub(args.topic, args.value)
    elif args.command == 'echo':
        topic_echo(args.topic)
    elif args.command == 'list':
        topic_list()
    else:
        raise NotImplementedError()


if __name__ == '__main__':
    main()
