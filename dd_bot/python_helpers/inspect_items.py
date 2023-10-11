import cv2
import numpy as np
import sys
from PIL import ImageGrab, Image


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


def apply_gaussian_blur(img, kernel_size=(1, 1)):
    """
    Apply Gaussian blur to the image.
    """
    return cv2.GaussianBlur(img, kernel_size, 0)


def non_max_suppression(boxes, overlapThresh):
    """
    Non-max suppression to remove overlapping bounding boxes.
    """
    if len(boxes) == 0:
        return []

    if boxes.dtype.kind == "i":
        boxes = boxes.astype("float")

    pick = []
    x1 = boxes[:, 0]
    y1 = boxes[:, 1]
    x2 = boxes[:, 2]
    y2 = boxes[:, 3]
    area = (x2 - x1 + 1) * (y2 - y1 + 1)
    idxs = np.argsort(y2)

    while len(idxs) > 0:
        last = len(idxs) - 1
        i = idxs[last]
        pick.append(i)
        suppress = [last]
        for pos in range(0, last):
            j = idxs[pos]
            xx1 = max(x1[i], x1[j])
            yy1 = max(y1[i], y1[j])
            xx2 = min(x2[i], x2[j])
            yy2 = min(y2[i], y2[j])
            w = max(0, xx2 - xx1 + 1)
            h = max(0, yy2 - yy1 + 1)
            overlap = float(w * h) / area[j]
            if overlap > overlapThresh:
                suppress.append(pos)
        idxs = np.delete(idxs, suppress)

    return boxes[pick].astype("int")


def template_matching(img, template_path, threshold=0.70):
    # Apply Gaussian blur to the screenshot (since img is now the image and not the path)
    img = apply_gaussian_blur(img)

    template = apply_gaussian_blur(cv2.imread(template_path))
    img_display = img.copy()

    emphasized_img = extract_goldish_color(img)
    emphasized_template = extract_goldish_color(template)

    w, h = template.shape[1], template.shape[0]
    res = cv2.matchTemplate(emphasized_img, emphasized_template, cv2.TM_CCOEFF_NORMED)
    loc = np.where(res >= threshold)

    #cv2.imshow("Emphasized Image", emphasized_img)
    #cv2.waitKey(0)

    detected_instances = 0
    boxes = []

    for pt in zip(*loc[::-1]):
        confidence = res[pt[1]][pt[0]]
        boxes.append([pt[0], pt[1], pt[0] + w, pt[1] + h])

    boxes = np.array(boxes)
    boxes = non_max_suppression(boxes, 0.5)

    for box in boxes:
        cv2.rectangle(img_display, (box[0], box[1]), (box[2], box[3]), (0, 255, 0), 2)
        print(f"{box[0]} {box[1]} {box[2]} {box[3]}")

        detected_instances += 1

    #cv2.imshow("Detected Icons", img_display)
    #cv2.waitKey(0)
    cv2.destroyAllWindows()

    return detected_instances


# Capture a screenshot using ImageGrab
screenshot = ImageGrab.grab()
screenshot_np = np.array(screenshot)  # Convert to numpy array
screenshot_bgr = cv2.cvtColor(screenshot_np, cv2.COLOR_RGB2BGR)  # Convert to BGR

# Run the function
template_path = "images/inspect_items.png"
template_matching(screenshot_bgr, template_path)