import cv2
import numpy as np
from PIL import ImageGrab
import time
import sys


# sys.argv[0] is the script name itself.
# sys.argv[1] will be "my_argument_value" if provided.

sensitive = False
cool = False

fast = False
super_fast = False

# If we want to wait longer before returning then use the "L" argument. Good for waiting for updates.
longer = False

if len(sys.argv) > 1 and not len(sys.argv) > 2:
    image_name = sys.argv[1]
elif len(sys.argv) > 2:
    image_name = sys.argv[1]
    for arg in sys.argv[2:4]:
        if arg.strip() == "S":
            sensitive = True
        elif arg.strip() == "C":
            cool = True
        elif arg.strip() == "F":
            fast = True
        elif arg.strip() == "SF":
            super_fast = True
        elif arg.strip() == "L":
            longer = True
else:
    image_name = "images/present_trader.png"

max_val = 0.00

tries = 0

if sensitive:
    limit = 0.98
elif cool:
    limit = 0.70
else:
    limit = 0.90

if fast:
    max_tries = 5
elif super_fast:
    max_tries = 2
elif longer:
    max_tries = 600
else:
    max_tries = 240

while max_val < limit:
    # If it has tried for 4 minutes then break
    if tries > max_tries:
        break
    # Capture a screenshot using ImageGrab
    screenshot = ImageGrab.grab()

    # Convert the screenshot to an OpenCV format
    main_image = np.array(screenshot)

    # Convert the screenshot image from BGR to RGB (OpenCV loads images in BGR by default)
    main_image = cv2.cvtColor(main_image, cv2.COLOR_BGR2RGB)

    # Load the template
    template = cv2.imread(image_name, cv2.IMREAD_COLOR)

    # Use template matching
    result = cv2.matchTemplate(main_image, template, cv2.TM_CCOEFF_NORMED)
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

    tries += 1
    time.sleep(1)
    # print(f"Certainty Score: {max_val:.2f}")


if tries < max_tries:
    # Get the top-left corner of the matched area
    top_left = max_loc
    bottom_right = (top_left[0] + template.shape[1], top_left[1] + template.shape[0])

    # Draw a rectangle around the matched object
    cv2.rectangle(main_image, top_left, bottom_right, (0, 255, 0), 2)

    # Display the result in a named window
    # window_name = "Detected Object"
    # cv2.namedWindow(window_name, cv2.WINDOW_NORMAL)
    # cv2.setWindowProperty(window_name, cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)
    # cv2.imshow(window_name, main_image)

    # Print the coordinates of the detected object directly
    print(top_left[0], top_left[1], bottom_right[0], bottom_right[1])

    # cv2.destroyAllWindows()
else:
    print("Could not detect")
