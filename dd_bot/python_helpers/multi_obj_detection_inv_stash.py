import cv2
import numpy as np
from PIL import ImageGrab, Image
import time
import sys
from imutils.object_detection import non_max_suppression

sensitive = False
cool = False
super_cool = False
fast = False
super_fast = False

# Will grayscale both the image and the screenshot if True.
grayscale = False

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
    image_name = "images/35_gold_pouch.png"

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

x_start, x_end = (
    1365,
    1880,
)  # Define the width interval where you want to perform template matching


# Preprocessing function
def preprocess(image):
    # Convert to grayscale
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
