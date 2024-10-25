from .. import MeasureTask
import psutil

class CpuTempMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 2000)

    def measurement(self):
        temps = psutil.sensors_temperatures(fahrenheit=False)
        current = temps["cpu_thermal"][0].current
        return [("TPU/OnBoard/CpuTemp", [current], "celsius")]
    
    
class CpuUsageMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 50)

    def measurement(self):
        cpu_usage = psutil.cpu_percent()
        return [("TPU/OnBoard/CpuUsage", [cpu_usage], "percent")]



class BrokerCpuUsageMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 100)
         with open("/var/run/mosquitto.pid", "r") as file:
            pid = int(file.readlines()[0])
            print("Pid is", pid)
            self.process = psutil.Process(pid)


    def measurement(self):
        broker_cpu_usage = self.process.cpu_percent()
        return [("TPU/OnBoard/BrokerCpuUsage", [broker_cpu_usage], "percent")]


class MemAvailMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 500)

    def measurement(self):
        mem_info = psutil.virtual_memory()
        mem_available = mem_info.available / (1024 * 1024)
        return [("TPU/OnBoard/MemAvailable", [mem_available], "MB")]


def main():
    ex1 = CpuTempMT()
    print(ex1.measurement())

    ex2 = CpuUsageMT()
    print(ex2.measurement())

    ex3 = BrokerCpuUsageMT()
    print(ex3.measurement())

    ex4 = MemAvailMT()
    print(ex4.measurement())


if __name__ == "__main__":
    main()
