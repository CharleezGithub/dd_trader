"""
Debugging / helper script for getting the coords of the mouse.
Not used by any other scripts
"""


import pyautogui
import time

try:
    while True:
        # Get the current mouse coordinates
        x, y = pyautogui.position()
        print(f"Cursor Coordinates: X={x}, Y={y}")
        time.sleep(1)  # Print every second
except KeyboardInterrupt:
    print("\nScript terminated.")
