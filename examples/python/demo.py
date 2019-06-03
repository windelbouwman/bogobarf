import logging

import bogobarf as bb
import asyncio


async def main():
    node = bb.Node('demo')
    await node.connect()

    # node.write_msg('bogobarf-python!'.encode())

    pub = node.publisher('/bla')
    # for x in range(10000):
    for x in range(10):
        await pub.write(x)

    # print('got back:', await node.read_msg())
    await node.disconnect()

print('Enter io loop')
asyncio.run(main())
print('Exit io loop')
