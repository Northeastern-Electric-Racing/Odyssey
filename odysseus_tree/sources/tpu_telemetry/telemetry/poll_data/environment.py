# open file to read error
from .. import MeasureTask


TEMP_SENSOR_PATH = '/sys/class/hwmon/hwmon2/temp1_input'
HUMIDITY_SENSOR_PATH = '/sys/class/hwmon/hwmon2/humidity1_input'
class EnvironmentMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 1000)

    def read_sensor_data(sensor_path):
        """
        Read the sensor path and send it
        """
        # try:
        with open(sensor_path, 'r') as f:
            sensor_data = int(f.read().strip()) / 1000  
            return sensor_data
        # except IOError:
        #     print("Error: Unable to read data from", sensor_path)
        #     return []

    def measurement(self):
        data = []

        temperature = self.read_sensor_data(TEMP_SENSOR_PATH)
        if float(temperature) is not None:
            data.append(("TPU/Environment/Temperature", [temperature], "Celsius"))

        humidity = self.read_sensor_data(HUMIDITY_SENSOR_PATH = '/sys/class/hwmon/hwmon2/humidity1_input')
        if float(humidity) is not None:
            data.append(("TPU/Environment/Humidity", [humidity], "%"))

        return data



def main():
    ex = EnvironmentMT()
    print(ex.measurement())


if __name__ == "__main__":
    main()
