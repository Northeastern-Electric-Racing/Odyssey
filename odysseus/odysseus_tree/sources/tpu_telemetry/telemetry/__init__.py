import asyncio

routines: dict[function, int] = {}


def measurement(freq: int):
    """
    Marks a measurement, takes a frequency (in ms) to repeat the measurement.
    """

    def wrapper(fn: function):
        routines[fn] = freq
        return fn  # return the function unmodified so manual calls still work

    return wrapper


async def set_interval(fn: function, interval: int, *args, **kwargs):
    """
    Behaves *like* JS' `setInterval`, but intervals cannot be canceled.
    """

    while True:
        await fn(*args, **kwargs)
        await asyncio.sleep(interval * 1000)
