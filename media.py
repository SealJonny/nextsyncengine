from exif import Image
import logging
import os
import helpers

logger = logging.getLogger(__name__)

IMAGE_FORMATS = [
    ".jpg",  # JPEG
    ".jpeg", # JPEG
    ".tiff", # Tagged Image File Format
    ".tif",  # Tagged Image File Format
]

def get_datetime_image(image_path):
    """
    extracts and returns the exif tags 'datetime_original' and 'datetime'
    """
    try:
        # open the file
        with open(image_path , "rb") as image_file:
            img  = Image(image_file)
        
        # extract the tags if it has some
        if img.has_exif:
            creation_date = helpers.convert_to_unix(img.get("datetime_original"))
            modification_date = helpers.convert_to_unix(img.get("datetime"))
            return creation_date, modification_date
    except FileNotFoundError as err:
        logger.error(err)

    return None, None

def get_datetime_non_image(video_path):
    """
    returns ctime and mtime from the file at 'video_path'
    """
    try:
        # raising a 'FileNotFoundError' if there is no file at 'video_path'
        if not os.path.isfile(video_path):
            raise FileNotFoundError(f"Extracting ctime and mtime from '{video_path}' failed")
        
        # extract ctime and mtime from file
        video_stats = os.stat(video_path)
        creation_date = int(video_stats.st_ctime)
        modification_date = int(video_stats.st_mtime)

        return creation_date, modification_date
    except FileNotFoundError as err:
        logger.error(err)
    return None, None


def get_datetime(media_path):
    """
    returns the creation_date and modification_date from the file at 'media_path'
    """
    # checking if file is supported by exif and using it if that's the case
    _, ext = os.path.splitext(media_path)
    for f in IMAGE_FORMATS:
        if ext == f:
            return get_datetime_image(media_path)
    return get_datetime_non_image(media_path)

#print(get_datetime("/home/sealjonny/test.mp4"))