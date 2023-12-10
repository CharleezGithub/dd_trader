"""
Multiple object detection script.
It detects multiple instances of an image and returns "prints" out the coordinates in a box like manner.
Coordiantes start at the top left of the screen at (0,0) and increase on the x axis when going right and on the y axis when going down.
Format: x_top_left y_top_left x_bottom_right y_bottom_right
Example:
321 43 421 54
42 542 54 734
450 341 654 504
"""


import cv2
import numpy as np
from PIL import ImageGrab
import time
import sys
from imutils.object_detection import non_max_suppression

sensitive = False
cool = False
super_cool = False
fast = False
super_fast = False
# Will grayscale both the image and the screenshot if True.
grayscale = True

crop = False

if len(sys.argv) > 1 and not len(sys.argv) > 2:
    image_name = sys.argv[1]
elif len(sys.argv) > 2:
    image_name = sys.argv[1]
    for arg in sys.argv[2:4]:
        if arg.strip() == "S":
            sensitive = True
        elif arg.strip() == "C":
            cool = True
        elif arg.strip() == "SC":
            super_cool = True
        elif arg.strip() == "F":
            fast = True
        elif arg.strip() == "SF":
            super_fast = True
        elif arg.strip() == "G":
            grayscale = True
        elif arg.strip() == "CR":
            crop = True
else:
    image_name = "temp_images/item/image.png"

max_val = 0.00

tries = 0

if sensitive:
    limit = 0.98
elif cool:
    limit = 0.70
elif super_cool:
    limit = 0.60
else:
    limit = 0.90

if fast:
    max_tries = 5
elif super_fast:
    max_tries = 2
else:
    max_tries = 240

while max_val < limit:
    if tries > max_tries:
        break

    screenshot = ImageGrab.grab()
    main_image = np.array(screenshot)
    main_image = cv2.cvtColor(main_image, cv2.COLOR_BGR2RGB)

    template = cv2.imread(image_name, cv2.IMREAD_COLOR)

    if grayscale:
        main_image = cv2.cvtColor(main_image, cv2.COLOR_RGB2GRAY)
        template = cv2.cvtColor(template, cv2.COLOR_BGR2GRAY)

    if crop:
        # The shape of an opencv image follows this format: Height (Rows), Width (Columns), Channels
        height, width = template.shape

        # 0.05 means 5% of the image will be cut off from every side of the image.
        # This can be recalibrated.
        # Smaller images should be cropped more drasticly as the in-game borders around the item do not change but only the size of the item
        # Bigger items will get scaled too far.
        if height < 70:
            crop_height = int(height * 0.15)
        else:
            crop_height = int(height * 0.05)
        
        if width < 70:
            crop_width = int(width * 0.15)
        else:
            crop_width = int(width * 0.05)

        # Format of cropping is: [Start height:End Height, Start width:End width]
        template = template[crop_height:-crop_height, crop_width:-crop_width]

    result = cv2.matchTemplate(main_image, template, cv2.TM_CCOEFF_NORMED)
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

    tries += 1
    time.sleep(1)

if tries < max_tries:
    threshold = limit
    loc = np.where(result >= threshold)

    # Store rectangles in array
    rectangles = []
    for pt in zip(*loc[::-1]):
        x1, y1 = pt[0], pt[1]
        x2, y2 = pt[0] + template.shape[1], pt[1] + template.shape[0]
        rectangles.append([x1, y1, x2, y2])

    # Apply non-max suppression to the bounding boxes
    rects = np.array(rectangles)
    pick = non_max_suppression(rects, probs=None, overlapThresh=0.65)

    for x1, y1, x2, y2 in pick:
        cv2.rectangle(main_image, (x1, y1), (x2, y2), (0, 255, 0), 2)
        print(f"{x1} {y1} {x2} {y2}")

    #window_name = "Detected Objects"
    #cv2.namedWindow(window_name, cv2.WINDOW_NORMAL)
    #cv2.setWindowProperty(window_name, cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)
    #cv2.imshow(window_name, main_image)

    #cv2.waitKey(0)
    #cv2.destroyAllWindows()
else:
    print("Could not detect")
