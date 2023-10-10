import cv2
import numpy as np
from PIL import ImageGrab
import time
import sys
from imutils.object_detection import non_max_suppression

sensitive = False

if len(sys.argv) > 1 and not len(sys.argv) > 2:
    image_name = sys.argv[1]
elif len(sys.argv) > 2:
    image_name = sys.argv[1]
    if sys.argv[2].strip() == "S":
        sensitive = True
else:
    image_name = "python_helpers/images/gold_fee_double_check.png"

max_val = 0.00

tries = 0

if sensitive:
    limit = 0.98
else:
    limit = 0.90

while max_val < limit:
    if tries > 240:
        break

    screenshot = ImageGrab.grab()
    main_image = np.array(screenshot)
    main_image = cv2.cvtColor(main_image, cv2.COLOR_BGR2RGB)

    template = cv2.imread(image_name, cv2.IMREAD_COLOR)

    result = cv2.matchTemplate(main_image, template, cv2.TM_CCOEFF_NORMED)
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

    tries += 1
    time.sleep(1)

if tries < 240:
    threshold = 0.8  
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

    for (x1, y1, x2, y2) in pick:
        cv2.rectangle(main_image, (x1, y1), (x2, y2), (0, 255, 0), 2)
        print(f"{x1} {y1} {x2} {y2}")
    
    window_name = "Detected Objects"
    cv2.namedWindow(window_name, cv2.WINDOW_NORMAL)
    cv2.setWindowProperty(window_name, cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)
    cv2.imshow(window_name, main_image)

    #cv2.waitKey(0)
    cv2.destroyAllWindows()
else:
    print("Could not detect")