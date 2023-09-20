import cv2
import numpy as np
import sys


def extract_goldish_color(img):
    # Convert the image to HSV space
    img_hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)

    # Define a range for the goldish yellow color (adjust these values as needed)
    """
    lower_gold = np.array([17, 36, 52])
    lower_gold = np.array([18, 41, 82])  # 0.81.
    lower_gold = np.array([15, 58, 133])  # 0.85.
    lower_gold = np.array(
        [15, 58, 200]
    )  # 0.92 but went from 7 to 2 instances detected.
    lower_gold = np.array([15, 58, 203])  # 0.92 but only 1 instance detected.
    lower_gold = np.array(
        [19, 70, 203]
    )  # 0.92 but visually only the key is left, everything else is black
    lower_gold = np.array([19, 70, 205])  # 0.96 but 3 detected instead of 4
    """
    lower_gold = np.array([19, 135, 205])  # 0.96 but 3 detected instead of 4
    # upper_gold = np.array([179, 218, 239])
    # upper_gold = np.array([255, 240, 249])
    """
    upper_gold = np.array([194, 225, 242])  # 0.71
    upper_gold = np.array([212, 240, 255])  # 0.71
    upper_gold = np.array([185, 220, 240])  # 0.85
    upper_gold = np.array([143, 195, 247])  # 0.86
    upper_gold = np.array([143, 155, 250])  # 0.87
    upper_gold = np.array([20, 155, 250])  # 0.92
    upper_gold = np.array([19, 155, 250])  # 0.92
    upper_gold = np.array([19, 255, 255])  # 0.93
    """
    upper_gold = np.array([20, 255, 255])  # 0.96 but 4 detectied instead of 1

    # Create a mask of goldish yellow regions
    mask = cv2.inRange(img_hsv, lower_gold, upper_gold)

    # Use the mask to extract the goldish yellow regions from the original image
    emphasized = cv2.bitwise_and(img, img, mask=mask)

    return emphasized


def template_matching(image_path, template_path, threshold=0.90):
    # Load the template and image
    template = cv2.imread(template_path)
    img = cv2.imread(image_path)
    img_display = img.copy()  # For displaying the result

    emphasized_img = extract_goldish_color(img)
    emphasized_template = extract_goldish_color(template)

    w, h = template.shape[1], template.shape[0]

    # Apply template matching using the emphasized images
    res = cv2.matchTemplate(emphasized_img, emphasized_template, cv2.TM_CCOEFF_NORMED)
    loc = np.where(res >= threshold)

    # Show the emphasized image
    cv2.imshow("Emphasized Image", emphasized_img)
    cv2.waitKey(0)

    detected_instances = 0
    coords = []

    # Iterate through the detected locations and draw rectangles
    for pt in zip(*loc[::-1]):
        confidence = res[pt[1]][pt[0]]
        cv2.rectangle(img_display, pt, (pt[0] + w, pt[1] + h), (0, 255, 0), 2)

        # Append the top-left and bottom-right corners
        coords.append([f"{pt[0]} {pt[1]} {pt[0] + w} {pt[1] + h}"])

        detected_instances += 1
        # print(f"Confidence: {confidence:.2f}")

    # Display the result with detections
    cv2.imshow("Detected Icons", img_display)
    cv2.waitKey(0)
    cv2.destroyAllWindows()

    print(detected_instances)
    for i in range(detected_instances):
        print(coords[i][0])

    return detected_instances


# sys.argv[0] is the script name itself.
# sys.argv[1] will be "my_argument_value" if provided.

if len(sys.argv) > 1:
    image_name = sys.argv[1]
else:
    image_name = "images/play.png"

# Run the function
image_name = "images/test2.png"
template_path = "images/inspect_items.png"
template_matching(image_name, template_path)
