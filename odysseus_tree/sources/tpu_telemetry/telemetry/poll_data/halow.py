from .. import BufferedCommand, MeasureTask, OneshotCommand

example_data = """BSSID         BW          TX bit rate        RX bit rate
=====================================================================
00:00:00:00:00:00     1M        0.33MBit/s(MCS 0)   0.33MBit/s(MCS 0)
=====================================================================
OK
"""
example_RSSI = """---------------------------------------------------
RSSI                             : 0
CS_Cnt                           : 54
PSDU_Succ                        : 0
MPDU_Rcv                         : 0
MPDU_Succ                        : 0
SNR                              : 0
---------------------------------------------------
OK"""


FETCH_THROUGHPUT_CMD = ["bmon", "-o", "format:fmt='$(attr:txrate:bytes) $(attr:rxrate:bytes)\n'", "-p", "bond0"]
FETCH_RATE_CMD = ["cli_app","show", "ap", "0"]
FETCH_RSSI_CMD = ["cli_app", "show", "stats", "simple_rx"]

class HalowThroughputMT(MeasureTask, BufferedCommand):
    def __init__(self):
         MeasureTask.__init__(self, 1000)
         BufferedCommand.__init__(self, FETCH_THROUGHPUT_CMD)

    def measurement(self):
        items = self.read()
        send_data = []
        for item in items:
            item = item.strip('\'').split(" ")
            data = [float(item[0].strip()), float(item[1].strip())]
            send_data.append(('TPU/HaLow/DataRate', data, 'kb/s'))
            
        return send_data


class HalowMCSMT(MeasureTask, OneshotCommand):
    def __init__(self):
         MeasureTask.__init__(self, 500)
         OneshotCommand.__init__(self, FETCH_RATE_CMD, 450)

    def measurement(self):
        out = self.read()
        send_data = []
        for line in out:
            data_line = line.splitlines()[2]
            parsed_data_ap = float(data_line.split()[5][:-1].strip())
            parsed_data_sta = float(data_line.split()[3][:-1].strip())
            send_data.append(("TPU/HaLow/ApMCS", [parsed_data_ap], "int"))
            send_data.append(("TPU/HaLow/StaMCS", [parsed_data_sta], "int"))

        return send_data


class HalowRSSIMT(MeasureTask, OneshotCommand):
    def __init__(self):
         MeasureTask.__init__(self, 500)
         OneshotCommand.__init__(self, FETCH_RSSI_CMD, 450)

    def measurement(self):
        out = self.read()
        send_data = []
        for line in out:
            split = line.splitlines()[1]
            data = float(split.split(":")[1].strip())
            send_data.append(("TPU/HaLow/RSSI", [data], "int"))
        return send_data




def main():
    ex1 = HalowThroughputMT()
    print(ex1.measurement())

    ex2 = HalowMCSMT()
    print(ex2.measurement())

    ex3 = HalowRSSIMT()
    print(ex3.measurement())


if __name__ == "__main__":
    main()
