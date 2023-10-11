import pytesseract
from PIL import Image, ImageFilter, ImageGrab

pytesseract.pytesseract.tesseract_cmd = r'C:\Program Files\Tesseract-OCR\tesseract.exe'

# Capture a screenshot using ImageGrab
image = ImageGrab.grab()

crop_box = (800, 279, 910, 301)

# Crop the image
image = image.crop(crop_box)

# Convert the image to grayscale
image = image.convert("L")

# Display the cropped and grayscale image
# image.show()

# Use Tesseract to do OCR on the image
custom_config = r"--oem 3 --psm 6 outputbase digits"  # OEM 3 is both standard and LSTM OCR, and PSM 6 assumes a single uniform block of text.
text = pytesseract.image_to_string(image, config=custom_config)

# Print the extracted text
if text.strip() == "":
    print("No text detected")
else:
    print(text.strip())
