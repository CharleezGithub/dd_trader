from PIL import Image
from PIL import ImageDraw, ImageFont
import requests
from io import BytesIO

async def stitch_images(user1_urls, user2_urls):
    """Stitch together images from the provided URLs for both users."""
    user1_images = [
        Image.open(BytesIO(requests.get(url).content)) for url in user1_urls
    ]
    user2_images = [
        Image.open(BytesIO(requests.get(url).content)) for url in user2_urls
    ]

    # Padding values (change these to adjust the space)
    arrow_padding = 50  # Added space on each side of the arrow
    side_padding = 20  # Added space on each side of the items

    # Determine max width and total height for each user's images
    max_width_user1 = max(img.width for img in user1_images) + 2 * side_padding
    max_width_user2 = max(img.width for img in user2_images) + 2 * side_padding
    total_height_user1 = sum(img.height for img in user1_images)
    total_height_user2 = sum(img.height for img in user2_images)

    # Create an arrow image with added padding and the same height as the tallest column
    max_height = max(total_height_user1, total_height_user2)
    arrow_image_width = 50 + 2 * arrow_padding
    arrow_image = Image.new("RGB", (arrow_image_width, max_height), color="white")
    draw = ImageDraw.Draw(arrow_image)
    font = ImageFont.truetype("arial.ttf", 50)
    draw.text((arrow_padding, (max_height - 35) // 2), "<--->", font=font, fill="black")

    # Create the final stitched image
    total_width = max_width_user1 + arrow_image.width + max_width_user2
    new_image = Image.new("RGB", (total_width, max_height), color="white")

    # Paste user1 images with side padding
    y_offset = (max_height - total_height_user1) // 2
    for img in user1_images:
        x_offset = side_padding
        new_image.paste(img, (x_offset, y_offset))
        y_offset += img.height

    # Paste arrow
    new_image.paste(arrow_image, (max_width_user1, 0))

    # Paste user2 images with side padding
    y_offset = (max_height - total_height_user2) // 2
    for img in user2_images:
        x_offset = max_width_user1 + arrow_image.width + side_padding
        new_image.paste(img, (x_offset, y_offset))
        y_offset += img.height

    buffer = BytesIO()
    new_image.save(buffer, "PNG")
    buffer.seek(0)

    return buffer