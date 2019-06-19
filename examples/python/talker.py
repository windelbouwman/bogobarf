
"""
This is a demo of how to publish values.
"""


import asyncio
from itertools import count
import bogobarf as bb


async def main():
    node = bb.Node('demo')
    await node.connect()
    pub = await node.publisher('/chatter')
    for x in count(1):
        value = f"Hello world {x}"
        print('Publishing:', value)
        await pub.write(value)
        await asyncio.sleep(1)
    await node.disconnect()

asyncio.run(main())
