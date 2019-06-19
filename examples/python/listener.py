""" Demo node which subscribes to a topic.
"""

import asyncio
import bogobarf as bb

async def on_chatter(value):
    print(f'/chatter: {value}')

async def main():
    node = bb.Node('demo')
    await node.connect()
    await node.subscribe('/chatter', on_chatter)
    while True:
        await asyncio.sleep(1)
    await node.disconnect()

asyncio.run(main())
