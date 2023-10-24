import cv2
import numpy as np
from PIL import ImageGrab, Image
import time
import sys
from imutils.object_detection import non_max_suppression

sensitive = False
cool = False

fast = False

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
else:
    image_name = "images/35_gold_pouch.png"

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

x_start, x_end = (
    1365,
    1880,
)  # Define the width interval where you want to perform template matching

# Preprocessing function
def preprocess(image):
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)


    blurred = cv2.GaussianBlur(gray, (3,3), 0)


    return blurred

try:
    while max_val < limit:
        if tries > max_tries:
            break

        screenshot = ImageGrab.grab()
        main_image_full = np.array(screenshot)
        main_image_full = cv2.cvtColor(main_image_full, cv2.COLOR_BGR2RGB)

        main_image = main_image_full[
            :, x_start:x_end
        ]  # Crop the image within the width interval

        main_image = preprocess(main_image)

        template = cv2.imread(image_name, cv2.IMREAD_COLOR)
        template = preprocess(template)

        result = cv2.matchTemplate(main_image, template, cv2.TM_CCOEFF_NORMED)
        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

        print(max_val)

        #pil = Image.fromarray(main_image)
        #pil.show()
        tries += 1
        time.sleep(1)


    if tries < max_tries:
        threshold = limit
        loc = np.where(result >= threshold)

        rectangles = []
        for pt in zip(*loc[::-1]):
            x1, y1 = pt[0] + x_start, pt[1]  # Add the x_offset to x1 coordinate
            x2, y2 = (
                pt[0] + template.shape[1] + x_start,
                pt[1] + template.shape[0],
            )  # Add the x_offset to x2 coordinate
            rectangles.append([x1, y1, x2, y2])

        rects = np.array(rectangles)
        pick = non_max_suppression(rects, probs=None, overlapThresh=0.65)

        for x1, y1, x2, y2 in pick:
            cv2.rectangle(
                main_image_full, (x1, y1), (x2, y2), (0, 255, 0), 2
            )  # Draw rectangle on the full image
            print(f"{x1} {y1} {x2} {y2}")

        # window_name = "Detected Objects"
        # cv2.namedWindow(window_name, cv2.WINDOW_NORMAL)
        # cv2.setWindowProperty(window_name, cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)
        # cv2.imshow(
        # window_name, main_image_full
        # )  # Show the full image with matched rectangles

        # cv2.waitKey(0)
        # cv2.destroyAllWindows()
    else:
        print("Could not detect")
except Exception as e:
    print("Error: \n", e)
