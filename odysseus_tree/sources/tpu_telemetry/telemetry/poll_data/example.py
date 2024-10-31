from .. import MeasureTask

class ExampleMT(MeasureTask):
    def __init__(self):
         MeasureTask.__init__(self, 1000)

    def measurement(self):
        return [
            ("TPU/Example/Data1", [114], "C"),
            ("TPU/Example/Data2", [1431242], "D"),
            ("TPU/Example/Data3", [1431242], "Q"),
            ("TPU/Example/Data4", [112343122], "X"),
            ("TPU/Example/Data5", [112341232], "M"),
            ("TPU/Example/Data6", [1413242], "W"),
        ]

def main():
    ex = ExampleMT()
    print(ex.measurement())


if __name__ == "__main__":
    main()
