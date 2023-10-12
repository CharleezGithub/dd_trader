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

if len(sys.argv) > 1 and not len(sys.argv) > 2:
    image_name = sys.argv[1]
elif len(sys.argv) > 2:
    image_name = sys.argv[1]
    if str(sys.argv[2].strip()) == "S":
        sensitive = True
    if str(sys.argv[2].strip()) == "C":
        cool = True
    if str(sys.argv[3].strip()) == "F":
        fast = True
else:
    image_name = "python_helpers/images/gold_fee_double_check.png"

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
    #print(f"Certainty Score: {max_val:.2f}")


if tries < max_tries:
    # Get the top-left corner of the matched area
    top_left = max_loc
    bottom_right = (top_left[0] + template.shape[1], top_left[1] + template.shape[0])

    # Draw a rectangle around the matched object
    cv2.rectangle(
        main_image, top_left, bottom_right, (0, 255, 0), 2
    )  # Change rectangle color to green for visibility

    # Display the result in a named window
    window_name = "Detected Object"
    cv2.namedWindow(window_name, cv2.WINDOW_NORMAL)
    # Set the window to fullscreen
    cv2.setWindowProperty(window_name, cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)

    cv2.imshow(window_name, main_image)

    # Get the screen coordinates of the window
    x, y, w, h = cv2.getWindowImageRect(window_name)

    # Calculate the screen coordinates of the detected object
    screen_top_left = (x + top_left[0], y + top_left[1])
    screen_bottom_right = (x + bottom_right[0], y + bottom_right[1])

    # Top left coords, Bottom right coords. x1 y1 x2 y2
    print(
        screen_top_left[0],
        screen_top_left[1],
        screen_bottom_right[0],
        screen_bottom_right[1],
    )

    # Print the certainty score (i.e., the maximum correlation coefficient)
    # print(f"Certainty Score: {max_val:.2f}")


    #cv2.waitKey(0)
    cv2.destroyAllWindows()
else:
    print("Could not detect")
