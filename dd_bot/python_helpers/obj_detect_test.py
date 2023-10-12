import cv2
import numpy as np
from PIL import ImageGrab
import time
import sys


# sys.argv[0] is the script name itself.
# sys.argv[1] will be "my_argument_value" if provided.

sensitive = False
cool = False

if len(sys.argv) > 1 and not len(sys.argv) > 2:
    image_name = sys.argv[1]
elif len(sys.argv) > 2:
    image_name = sys.argv[1]
    if str(sys.argv[2].strip()) == "S":
        sensitive = True
    if str(sys.argv[2].strip()) == "C":
        cool = True
else:
    image_name = "images/play.png"
    image_name = "python_helpers/images/image.png"

max_val = 0.00

# Make a timeout

tries = 0

if sensitive:
    limit = 0.98
elif cool:
    limit = 0.70
else:
    limit = 0.90
while max_val < limit:
    print("hello")
    if tries > 240:
        break
    # Capture a screenshot using ImageGrab
    screenshot = ImageGrab.grab()
    main_image = np.array(screenshot)
    main_image = cv2.cvtColor(main_image, cv2.COLOR_BGR2RGB)

    # Load the template
    template = cv2.imread(image_name, cv2.IMREAD_COLOR)
    (tH, tW) = template.shape[:2]

    # loop over the scales of the image
    found = None
    for scale in np.linspace(0.2, 1.0, 20)[::-1]:
        # resize the image according to the scale, and keep track of the ratio
        resized = cv2.resize(main_image, (int(main_image.shape[1] * scale), int(main_image.shape[0] * scale)))
        r = main_image.shape[1] / float(resized.shape[1])

        # if the resized image is smaller than the template, then break from the loop
        if resized.shape[0] < tH or resized.shape[1] < tW:
            break

        # detect edges in the resized, grayscale image and apply template matching
        result = cv2.matchTemplate(resized, template, cv2.TM_CCOEFF_NORMED)
        (_, maxVal, _, maxLoc) = cv2.minMaxLoc(result)

        # if we have found a new maximum correlation value, then update the corresponding variable
        if found is None or maxVal > found[0]:
            found = (maxVal, maxLoc, r)

    # unpack the found tuple and compute the (x, y) coordinates of the bounding box
    (_, maxLoc, r) = found
    (startX, startY) = (int(maxLoc[0] * r), int(maxLoc[1] * r))
    (endX, endY) = (int((maxLoc[0] + tW) * r), int((maxLoc[1] + tH) * r))

    # draw a bounding box around the detected result
    cv2.rectangle(main_image, (startX, startY), (endX, endY), (0, 255, 0), 2)

    # Get the screen coordinates of the window
    x, y, w, h = cv2.getWindowImageRect("Detected Object")

    # Calculate the screen coordinates of the detected object
    screen_top_left = (x + startX, y + startY)
    screen_bottom_right = (x + endX, y + endY)

    print(screen_top_left[0], screen_top_left[1], screen_bottom_right[0], screen_bottom_right[1])

    # show the detection and the confidence level
    print(f"Certainty Score: {found[0]:.2f}")

    tries += 1
    time.sleep(1)


if tries < 240:
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
    print(f"Certainty Score: {max_val:.2f}")


    cv2.waitKey(0)
    cv2.destroyAllWindows()
else:
    print("Could not detect")
