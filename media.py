import logging
import os
import helpers
import subprocess
import re

logger = logging.getLogger(__name__)

exiftool = "/usr/local/bin/exiftool-amd64-glibc"

IMAGE_FORMATS = [
    ".jpg", ".jpeg", ".tif", ".tiff", ".gif", ".bmp", ".png", ".ppm", 
    ".pgm", ".pbm", ".pnm", ".webp", ".heif", ".heic", ".jp2", ".j2k", 
    ".jpf", ".jpx", ".jpm", ".mj2", ".ico", ".cr2", ".cr3", ".nef", 
    ".nrw", ".orf", ".raf", ".arw", ".rw2", ".dng", ".sr2", ".3fr", 
    ".rwl", ".mrw", ".raw", ".pef", ".iiq", ".k25", ".kc2", ".erf", 
    ".srw", ".x3f", ".svg"
]

VIDEO_FORMATS = [
    ".mp4", ".mov", ".avi", ".mkv", ".3gp", ".3g2", ".wmv", ".asf", 
    ".flv", ".f4v", ".swf", ".m2ts", ".mts", ".m2t", ".ts", ".mxf", 
    ".mpg", ".mpeg", ".mpe", ".mpv", ".m4v", ".m4p", ".rm", ".rmvb", 
    ".webm", ".ogv", ".ogg", ".ogx", ".dv", ".dif", ".m2v", ".qt", 
    ".mjpg", ".mj2", ".gif", ".mov"
]

SUPPORTED_FORMATS = IMAGE_FORMATS + VIDEO_FORMATS

def get_datetime_media(media_path):
    """
    extracts and returns the creation and modify date as unix timestamps using the exiftool binary
    """
    #checking if the file exists and raising FileNotFoundError if check returns False
    try:
        if not os.path.isfile(media_path):
            raise FileNotFoundError(f"The provided path '{media_path}' does not point to a file")
    except FileNotFoundError as err:
        logger.error(err)
        return None, None
    # extract modifiy and creation date with exiftool binary. It searches in multiple tags and returns the first with a value
    ctime_cmd = f"{exiftool} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTimeOriginal -DateCreated -CreateDate -FileCreateDate '{media_path}' | head -n 1"
    mtime_cmd = f"{exiftool} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTime -ModifyDate -FileModifyDate '{media_path}' | head -n 1"

    result_ctime = subprocess.run(ctime_cmd, capture_output=True, text=True, shell=True, check=True)
    result_mtime = subprocess.run(mtime_cmd, capture_output=True, text=True, shell=True, check=True)

    # check if exiftool could extract the ctime and mtime, if not try again with get_datetime_no_media() 
    ctime_str = result_ctime.stdout
    if ctime_str == '':
        return get_datetime_non_media(media_path)

    mtime_str = result_mtime.stdout
    if mtime_str == '':
        return get_datetime_non_media(media_path)
    
    # remove trailing \n from ctime and mtime string
    ctime_str = ctime_str[:len(ctime_str) - 1]
    mtime_str = mtime_str[:len(mtime_str) - 1]

    return helpers.convert_to_unix(ctime_str), helpers.convert_to_unix(mtime_str)

def get_datetime_non_media(video_path):
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
    for f in SUPPORTED_FORMATS:
        if ext == f:
            return get_datetime_media(media_path)
    return get_datetime_non_media(media_path)