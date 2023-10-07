import cv2
import numpy as np
from PIL import ImageGrab
import time
import sys


# sys.argv[0] is the script name itself.
# sys.argv[1] will be "my_argument_value" if provided.

if len(sys.argv) > 1:
    image_name = sys.argv[1]
else:
    image_name = "images/play.png"
    image_name = "python_helpers/images/gold_fee4.png"

max_val = 0.00

# Make a timeout

tries = 0

# Lowering the threshold slightly
match_threshold = 0.85  

# Multi-scale template matching: considering scales from 80% to 120%
scales = np.linspace(0.8, 1.2, 5)  

while max_val < match_threshold:
    print("hello")
    if tries > 240:
        break
    
    screenshot = ImageGrab.grab()
    main_image = np.array(screenshot)
    main_image = cv2.cvtColor(main_image, cv2.COLOR_BGR2RGB)
    
    template = cv2.imread(image_name, cv2.IMREAD_COLOR)
    
    # Apply Gaussian blur to the template and main image
    main_image = cv2.GaussianBlur(main_image, (5, 5), 0)
    template = cv2.GaussianBlur(template, (5, 5), 0)
    
    for scale in scales:
        # Resize the template according to the scale
        resized_template = cv2.resize(template, (int(template.shape[1] * scale), int(template.shape[0] * scale)))
        
        result = cv2.matchTemplate(main_image, resized_template, cv2.TM_CCOEFF_NORMED)
        _, max_val, _, max_loc = cv2.minMaxLoc(result)
        
        if max_val > match_threshold:
            break  # If a match is found, exit the loop early
    
    tries += 1
    time.sleep(1)
    print(f"Certainty Score: {max_val:.2f}")


    cv2.waitKey(0)
    cv2.destroyAllWindows()
else:
    print("Could not detect")
