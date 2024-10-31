from .. import BufferedCommand, MeasureTask


# read using can-utils canbusload, ensure to change bitrate accordingly
FETCH_CMD = ["canbusload","can0@500000"]
class CanMT(MeasureTask, BufferedCommand):
    def __init__(self):
         MeasureTask.__init__(self, 1000)
         BufferedCommand.__init__(self, FETCH_CMD)

    def measurement(self):
        items = self.read()
        # filter out only newlines as list members
        items = filter(lambda x: x != '\n', items)
        send_data = []
        for item in items:
            item = item.strip('\'').strip().split(" ")
            # get rid of empty strings in list
            item = list(filter(None, item))
            # convert percentage to decimal
            send_data.append(('TPU/Can/BusUtil', [float(item[5].strip().strip("%")) / 100], '%'))
            send_data.append(('TPU/Can/BitsUsed', [float(item[2].strip())], 'bits'))
        
        return send_data
        


def main():
    ex = CanMT()
    print(ex.measurement())


if __name__ == "__main__":
    main()
