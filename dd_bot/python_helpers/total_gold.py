import pytesseract
from PIL import Image, ImageFilter

# Open the image file
image = Image.open("total_gold_test9.png")

# Convert the image to grayscale
image = image.convert("L")

# Display the grayscale image
image.show()

# Perform OCR using PyTesseract
text = pytesseract.image_to_string(image)

# Print the extracted text
print(text.strip())
