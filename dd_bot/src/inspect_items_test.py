import cv2
import numpy as np


def template_matching(image_path, template_path, threshold=0.5):
    # Load the template and image
    template = cv2.imread(template_path, cv2.IMREAD_GRAYSCALE)
    img = cv2.imread(image_path, cv2.IMREAD_GRAYSCALE)
    img_display = cv2.imread(image_path)  # For displaying the result

    w, h = template.shape[::-1]

    # Apply template matching
    res = cv2.matchTemplate(img, template, cv2.TM_CCOEFF_NORMED)
    loc = np.where(res >= threshold)

    detected_instances = 0

    # Show the original image first
    cv2.imshow("Original Image", img)
    cv2.waitKey(0)

    # Iterate through the detected locations and draw rectangles
    for pt in zip(*loc[::-1]):
        confidence = res[pt[1]][pt[0]]
        cv2.rectangle(img_display, pt, (pt[0] + w, pt[1] + h), (0, 255, 0), 2)
        detected_instances += 1
        print(f"Confidence: {confidence:.2f}")

    # Display the result with detections
    cv2.imshow("Detected Icons", img_display)
    cv2.waitKey(0)
    cv2.destroyAllWindows()

    print(f"Detected {detected_instances} instances of the icon.")

    return detected_instances


# Test the function
image_path = "test2.png"
template_path = "inspect_items.png"
template_matching(image_path, template_path)
