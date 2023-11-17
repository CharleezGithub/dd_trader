import cv2
from PIL import ImageGrab
import numpy as np
# Capture a screenshot using ImageGrab
screenshot = ImageGrab.grab()

# Convert the screenshot to an OpenCV format
main_image = np.array(screenshot)

main_image = cv2.cvtColor(main_image, cv2.COLOR_RGB2GRAY)
template = "test_info.png"
template = cv2.imread(template, 0)
# Load the template and the screenshot in grayscale

# Initialize ORB detector
orb = cv2.ORB_create()

# Find keypoints and descriptors
kp1, des1 = orb.detectAndCompute(template, None)
kp2, des2 = orb.detectAndCompute(main_image, None)

# Create BFMatcher and match descriptors
bf = cv2.BFMatcher(cv2.NORM_HAMMING, crossCheck=True)
matches = bf.match(des1, des2)

# Define a threshold for the distance (you'll need to determine the right value here)
distance_threshold = 9  # Example value

# Filter matches based on the distance threshold
good_matches = [m for m in matches if m.distance < distance_threshold]

# Draw matches
matched_image = cv2.drawMatches(template, kp1, main_image, kp2, good_matches, None, flags=2)

# Show the result
cv2.imshow('Good Matches', matched_image)
cv2.waitKey(0)
cv2.destroyAllWindows()
