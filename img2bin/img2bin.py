import sys
import struct
from PIL import Image


pixel_format = struct.Struct("BBB")


def main():
    if len(sys.argv) < 3:
        print("usage: python img2bin.py <image file> <bin file>")
        return 1
    
    img = Image.open(sys.argv[1])
    width, height = img.size

    with open(sys.argv[2], 'wb') as file:
        for y in range(0, height):
            for x in range(0, width):
                pixel = img.getpixel((x, y))
                file.write(pixel_format.pack(pixel[0], pixel[1], pixel[2]))


if __name__ == '__main__':
    main()
