
import asyncio
import logging

import bogobarf as bb


async def main():
    node = bb.Node('demo')
    await node.connect()

    # node.write_msg('bogobarf-python!'.encode())

    pub = node.publisher('/bla')
    # for x in range(10000):
    for x in range(10):
        msg = {
            'foo': x,
            'blaa': [1,2,3]
        }
        await pub.write(msg)

    # print('got back:', await node.connection.read_msg())
    await node.disconnect()

logging.basicConfig(level=logging.DEBUG)
logging.info('Enter io loop')
asyncio.run(main())
logging.info('Exit io loop')
