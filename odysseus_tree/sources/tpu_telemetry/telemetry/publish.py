import asyncio
import math
import signal
import time


from telemetry.poll_data import gps_data, halow, on_board, can
from . import (
    MTCommand,
    MeasureTask,
    server_data_pb2
)
from gmqtt import Client as MQTTClient

TASKS = []

# initialize connection
client = MQTTClient("tpu-publisher")
STOP = asyncio.Event()


def on_connect(client, flags, rc, properties):
    print("Connected")


def on_disconnect(client, packet, exc=None):
    print("Disconnected")


def ask_exit(*args):
    for task in TASKS:
        if isinstance(task, MTCommand):
            task.deinit()
    STOP.set()
    


def publish_data(topic, message_data):
    # send the data
    client.publish(topic, message_data)


async def interval(task: MeasureTask):
    async for result in task.set_interval(STOP):
        time_us = time.time() * 1000000
        # process each tuple
        for packet in result:
            data = server_data_pb2.ServerData()
            topic, values, data.unit = packet
            data.time_us = int(time_us)
            # process the data values
            for val in values:
                data.values.append(val)
            else:
                message_data = data.SerializeToString()
                publish_data(topic, message_data)
            

async def run(host):
    await client.connect(host, 1883)
    client.on_connect = on_connect
    client.on_disconnect = on_disconnect


    # ADD TASKS HERE for more measurements
    TASKS = [ # example.ExampleMT(), # uncomment so example data is sent
             can.CanMT(),
             # environment.EnvironmentMT() # commented out bc sensor is currently broken
             halow.HalowThroughputMT(),
             halow.HalowMCSMT(),
             halow.HalowRSSIMT(),
             on_board.CpuTempMT(),
             on_board.CpuUsageMT(),
             on_board.BrokerCpuUsageMT(),
             on_board.MemAvailMT(),
             gps_data.GpsMT()
             ]

    stagger = 1 / len(TASKS)
    for task in TASKS:

        # if task is of type BufferedCommand, register its thread
        if isinstance(task, MTCommand):
            task.get_thread().start()

        # should not be awaited, this just gets run parallely along with other intervals.
        asyncio.create_task(interval(task))
        await asyncio.sleep(stagger)

    await STOP.wait()

def main():
    loop = asyncio.new_event_loop()

    host = "localhost"

    loop.add_signal_handler(signal.SIGINT, ask_exit)
    loop.add_signal_handler(signal.SIGTERM, ask_exit)

    loop.run_until_complete(run(host))


if __name__ == "__main__":
    main()
