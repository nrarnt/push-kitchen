"""Makes the game's sound effects in assets/sounds.

Run from the project folder:  python3 tools/make_sounds.py
Uses only Python's standard library.

Every sound is built from sine waves and noise. To use your own sounds
instead, replace any of the WAV files with one of the same name.
"""

import math
import random
import struct
import wave
from pathlib import Path

RATE = 44100
OUT = Path(__file__).resolve().parent.parent / "assets" / "sounds"

random.seed(7)  # the same "random" noise every time, so the files do not change


def silence(seconds):
    return [0.0] * int(RATE * seconds)


def sweep(start_hz, end_hz, seconds, volume, decay=6.0):
    """A sine wave gliding from one pitch to another, fading out."""
    samples = []
    phase = 0.0
    count = int(RATE * seconds)
    for i in range(count):
        t = i / count
        phase += 2 * math.pi * (start_hz + (end_hz - start_hz) * t) / RATE
        samples.append(math.sin(phase) * volume * math.exp(-decay * t))
    return samples


def noise(seconds, volume, decay=6.0, smooth=0.0):
    """Hiss, fading out. `smooth` near 1 makes it duller, 0 leaves it sharp."""
    samples = []
    last = 0.0
    count = int(RATE * seconds)
    for i in range(count):
        t = i / count
        last = smooth * last + (1 - smooth) * random.uniform(-1, 1)
        samples.append(last * volume * math.exp(-decay * t))
    return samples


def mix(*sounds):
    """Plays several sounds at the same time."""
    length = max(len(sound) for sound in sounds)
    return [sum(sound[i] for sound in sounds if i < len(sound)) for i in range(length)]


def soften_ends(samples, seconds=0.003):
    """Fades the very start and end, so the sound does not click."""
    edge = int(RATE * seconds)
    for i in range(min(edge, len(samples))):
        samples[i] *= i / edge
        samples[-1 - i] *= i / edge
    return samples


def save(name, samples):
    OUT.mkdir(parents=True, exist_ok=True)
    samples = soften_ends(samples)
    with wave.open(str(OUT / f"{name}.wav"), "wb") as file:
        file.setnchannels(1)
        file.setsampwidth(2)
        file.setframerate(RATE)
        clipped = (max(-1.0, min(1.0, sample)) for sample in samples)
        file.writeframes(b"".join(struct.pack("<h", int(sample * 32767)) for sample in clipped))


def bell(hz, seconds, volume):
    return mix(sweep(hz, hz, seconds, volume, decay=5.0), sweep(hz * 2, hz * 2, seconds, volume * 0.3, decay=8.0))


def knock():
    return mix(noise(0.03, 0.35, decay=5.0, smooth=0.5), sweep(300, 150, 0.04, 0.3))


if __name__ == "__main__":
    save("step", mix(noise(0.05, 0.12, decay=7.0, smooth=0.85), sweep(200, 150, 0.05, 0.12)))
    save("push", mix(sweep(170, 105, 0.13, 0.4, decay=4.0), noise(0.13, 0.08, decay=5.0, smooth=0.7)))
    save("bump", sweep(110, 65, 0.1, 0.45, decay=5.0))
    save("chop", knock() + silence(0.05) + knock() + silence(0.05) + knock())
    save("sizzle", noise(0.5, 0.22, decay=3.5, smooth=0.15))
    save("combine", sweep(520, 780, 0.07, 0.3, decay=2.0) + sweep(780, 1040, 0.1, 0.3, decay=3.0))

    # Four notes going up, each ringing on under the next.
    notes = [523.25, 659.25, 783.99, 1046.5]
    save("solved", mix(*(silence(0.11 * i) + bell(hz, 0.6, 0.22) for i, hz in enumerate(notes))))

    # Last, so that adding it left the noise in the files above unchanged.
    save("bin", mix(sweep(260, 70, 0.2, 0.4, decay=4.0), noise(0.2, 0.12, decay=6.0, smooth=0.6)))

    # A long hiss over a falling tone: the chef on the stove.
    save("burnt", mix(noise(0.7, 0.3, decay=3.0, smooth=0.1), sweep(420, 90, 0.5, 0.3, decay=3.0)))

    # Two short, high chirps: a mouse that has found something to eat.
    chirp = sweep(2400, 3300, 0.06, 0.25, decay=2.0)
    save("squeak", chirp + silence(0.04) + chirp)
    # A falling whistle over a soft hiss: the chef sliding on grease.
    save("slip", mix(sweep(900, 320, 0.22, 0.22, decay=2.5), noise(0.22, 0.08, decay=4.0, smooth=0.6)))

    print(f"sounds written to {OUT}")
