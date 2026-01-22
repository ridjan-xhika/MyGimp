#!/usr/bin/env python3
"""Generate placeholder icons for the GIMP application."""

from PIL import Image, ImageDraw, ImageFont

def create_icon(name: str, size: int, bg_color, draw_func):
    """Create an icon with the given specifications."""
    img = Image.new('RGBA', (size, size), (255, 255, 255, 0))
    draw = ImageDraw.Draw(img)
    
    # Draw background
    draw.rectangle([(0, 0), (size - 1, size - 1)], fill=bg_color, outline=(100, 100, 100, 255))
    
    # Draw custom content
    draw_func(draw, size)
    
    img.save(f'assets/{name}.png')
    print(f'✓ Created {name}.png ({size}x{size})')

# Helper function to draw simple shapes
def draw_rect(draw, x1, y1, x2, y2, fill, outline=None):
    draw.rectangle([(x1, y1), (x2, y2)], fill=fill, outline=outline)

def draw_circle(draw, cx, cy, r, fill, outline=None):
    draw.ellipse([(cx - r, cy - r), (cx + r, cy + r)], fill=fill, outline=outline)

def draw_line(draw, x1, y1, x2, y2, color, width=1):
    draw.line([(x1, y1), (x2, y2)], fill=color, width=width)

# Icon creation functions
def grayscale_icon(draw, size):
    """Grayscale filter icon - desaturated color wheel."""
    center = size // 2
    radius = size // 4
    
    # Draw grayscale gradient circles (representing desaturation)
    colors = [
        (200, 200, 200, 255),
        (150, 150, 150, 255),
        (100, 100, 100, 255),
    ]
    for i, color in enumerate(colors):
        r = radius - (i * 3)
        if r > 0:
            draw_circle(draw, center, center, r, fill=color, outline=(50, 50, 50, 255))

def brightness_icon(draw, size):
    """Brightness/Contrast icon - sun or lightbulb."""
    center = size // 2
    # Draw a simple sun/brightness symbol
    radius = size // 4
    
    # Central circle (bulb or sun)
    draw_circle(draw, center, center, radius, fill=(255, 255, 100, 255), outline=(200, 150, 0, 255))
    
    # Add rays around it to indicate brightness
    ray_len = radius + size // 8
    ray_start = radius + 2
    for angle_idx in range(8):
        angle = (angle_idx * 45) * 3.14159 / 180
        x_off = int((ray_start + 3) * 3.14159 / angle_idx + 1)
        y_offset = int((ray_start + 3) * 3.14159 / (angle_idx + 1))
        
        # Simplified rays
        draw.rectangle([(center - 1, center - ray_len), (center + 1, center - ray_start)], 
                      fill=(255, 200, 0, 255))

def blur_icon(draw, size):
    """Blur filter icon - blurred shapes."""
    center = size // 2
    half = size // 2
    
    # Draw blurred-looking overlapping circles
    colors = [(100, 150, 200, 200), (150, 100, 200, 180), (200, 150, 100, 180)]
    offsets = [(-4, -4), (0, 4), (4, 0)]
    
    for color, (ox, oy) in zip(colors, offsets):
        draw_circle(draw, center + ox, center + oy, size // 6, fill=color, outline=(50, 50, 100, 200))

def invert_icon(draw, size):
    """Invert colors icon - split black and white."""
    half = size // 2
    
    # Left half black, right half white
    draw.rectangle([(0, 0), (half - 1, size - 1)], fill=(0, 0, 0, 255))
    draw.rectangle([(half, 0), (size - 1, size - 1)], fill=(255, 255, 255, 255))
    
    # Add diagonal arrow to show inversion
    arrow_color = (150, 150, 150, 255)
    draw_line(draw, half - 6, size // 2 - 4, half + 6, size // 2 + 4, arrow_color, 2)
    draw_line(draw, half + 4, size // 2 + 2, half + 6, size // 2 + 4, arrow_color, 2)

def select_rect_icon(draw, size):
    """Rectangle selection tool icon."""
    margin = size // 6
    draw.rectangle([(margin, margin), (size - margin - 1, size - margin - 1)], 
                   fill=None, outline=(100, 150, 255, 255), width=2)
    # Add selection indicator corners
    corner_size = 4
    corners = [(margin, margin), (size - margin - corner_size, margin), 
               (margin, size - margin - corner_size), (size - margin - corner_size, size - margin - corner_size)]
    for cx, cy in corners:
        draw.rectangle([(cx, cy), (cx + corner_size, cy + corner_size)], fill=(100, 150, 255, 255))

def select_ellipse_icon(draw, size):
    """Ellipse selection tool icon."""
    margin = size // 6
    draw.ellipse([(margin, margin), (size - margin - 1, size - margin - 1)], 
                 fill=None, outline=(100, 150, 255, 255), width=2)

def select_lasso_icon(draw, size):
    """Lasso selection tool icon - wavy/curved line."""
    center = size // 2
    # Draw a wavy lasso-like path
    points = []
    for i in range(size):
        y = (i * 0.5) % size
        x = center + int(5 * ((i % 20) / 10 - 1))
        if 0 <= x < size:
            points.append((x, int(y)))
    
    if len(points) > 1:
        for i in range(len(points) - 1):
            draw_line(draw, points[i][0], points[i][1], points[i+1][0], points[i+1][1], 
                     (100, 150, 255, 255), 2)

def shape_rect_icon(draw, size):
    """Rectangle shape tool icon."""
    margin = size // 5
    draw.rectangle([(margin, margin), (size - margin - 1, size - margin - 1)], 
                   fill=(150, 200, 255, 180), outline=(50, 100, 200, 255), width=2)

def shape_ellipse_icon(draw, size):
    """Ellipse shape tool icon."""
    margin = size // 5
    draw.ellipse([(margin, margin), (size - margin - 1, size - margin - 1)], 
                 fill=(150, 200, 255, 180), outline=(50, 100, 200, 255), width=2)

def shape_line_icon(draw, size):
    """Line shape tool icon."""
    margin = size // 5
    draw_line(draw, margin, size - margin, size - margin, margin, (50, 100, 200, 255), 3)
    # Add arrow head
    draw_line(draw, size - margin, margin, size - margin - 6, margin + 6, (50, 100, 200, 255), 2)
    draw_line(draw, size - margin, margin, size - margin - 6, margin - 6, (50, 100, 200, 255), 2)

# Create all missing icons
if __name__ == '__main__':
    print("Generating placeholder icons...")
    
    create_icon('grayscale', 227, (200, 200, 200, 255), grayscale_icon)
    create_icon('brightness', 512, (255, 255, 200, 255), brightness_icon)
    create_icon('blur', 360, (180, 200, 220, 255), blur_icon)
    create_icon('invert', 512, (128, 128, 128, 255), invert_icon)
    create_icon('select_rect', 512, (200, 220, 255, 255), select_rect_icon)
    create_icon('select_ellipse', 512, (200, 220, 255, 255), select_ellipse_icon)
    create_icon('select_lasso', 512, (200, 220, 255, 255), select_lasso_icon)
    create_icon('shape_rect', 512, (220, 240, 255, 255), shape_rect_icon)
    create_icon('shape_ellipse', 512, (220, 240, 255, 255), shape_ellipse_icon)
    create_icon('shape_line', 512, (220, 240, 255, 255), shape_line_icon)
    
    print("\n✓ All icons generated successfully!")
