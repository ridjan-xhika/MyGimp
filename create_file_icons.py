from PIL import Image, ImageDraw

def create_import_icon(size=256):
    """Import icon - folder with arrow pointing in"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Folder shape
    folder_color = (100, 150, 255, 255)  # Blue folder
    folder_y = size // 3
    folder_h = size // 2
    folder_w = int(size * 0.7)
    folder_x = (size - folder_w) // 2
    
    # Folder tab
    tab_w = folder_w // 3
    draw.rectangle([folder_x, folder_y - size//8, folder_x + tab_w, folder_y], fill=folder_color)
    # Main folder body
    draw.rectangle([folder_x, folder_y, folder_x + folder_w, folder_y + folder_h], fill=folder_color)
    
    # Arrow pointing down-left into folder
    arrow_color = (255, 255, 255, 255)
    arrow_size = size // 4
    cx = folder_x + folder_w // 2
    cy = folder_y + folder_h // 3
    
    # Arrow shaft (from top-right to center)
    shaft_width = size // 20
    draw.line([cx + arrow_size//2, cy - arrow_size//2, cx, cy], fill=arrow_color, width=shaft_width)
    
    # Arrow head pointing down-left
    head_size = size // 8
    draw.polygon([
        (cx, cy),
        (cx + head_size, cy - head_size//2),
        (cx + head_size//2, cy - head_size)
    ], fill=arrow_color)
    
    return img

def create_export_icon(size=256):
    """Export icon - folder with arrow pointing out"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Folder shape
    folder_color = (255, 150, 100, 255)  # Orange folder
    folder_y = size // 3
    folder_h = size // 2
    folder_w = int(size * 0.7)
    folder_x = (size - folder_w) // 2
    
    # Folder tab
    tab_w = folder_w // 3
    draw.rectangle([folder_x, folder_y - size//8, folder_x + tab_w, folder_y], fill=folder_color)
    # Main folder body
    draw.rectangle([folder_x, folder_y, folder_x + folder_w, folder_y + folder_h], fill=folder_color)
    
    # Arrow pointing up-right out of folder
    arrow_color = (255, 255, 255, 255)
    arrow_size = size // 4
    cx = folder_x + folder_w // 2
    cy = folder_y + folder_h // 3
    
    # Arrow shaft (from center to top-right)
    shaft_width = size // 20
    draw.line([cx, cy, cx + arrow_size//2, cy - arrow_size//2], fill=arrow_color, width=shaft_width)
    
    # Arrow head pointing up-right
    head_size = size // 8
    draw.polygon([
        (cx + arrow_size//2, cy - arrow_size//2),
        (cx + arrow_size//2 - head_size//2, cy - arrow_size//2 + head_size),
        (cx + arrow_size//2 - head_size, cy - arrow_size//2 + head_size//2)
    ], fill=arrow_color)
    
    return img

def create_save_icon(size=256):
    """Save icon - floppy disk"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Floppy disk body
    disk_color = (100, 100, 255, 255)  # Blue
    margin = size // 6
    
    # Main disk rectangle
    draw.rectangle([margin, margin, size - margin, size - margin], fill=disk_color)
    
    # Label area (white rectangle at top)
    label_h = size // 4
    draw.rectangle([margin + size//16, margin + size//16, size - margin - size//16, margin + label_h], fill=(255, 255, 255, 255))
    
    # Metal shutter at bottom
    shutter_color = (180, 180, 180, 255)
    shutter_y = size - margin - size//5
    draw.rectangle([margin + size//12, shutter_y, size - margin - size//12, size - margin - size//16], fill=shutter_color)
    
    # Shutter lines
    line_color = (100, 100, 100, 255)
    for i in range(3):
        y = shutter_y + (size//16) + i * (size//25)
        draw.line([margin + size//10, y, size - margin - size//10, y], fill=line_color, width=2)
    
    return img

# Generate icons
print("Creating file operation icons...")
create_import_icon().save('Gimp/assets/import.png')
print("✓ Created import.png")

create_export_icon().save('Gimp/assets/export.png')
print("✓ Created export.png")

create_save_icon().save('Gimp/assets/save.png')
print("✓ Created save.png")

print("\nAll file operation icons created successfully!")
