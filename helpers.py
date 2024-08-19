import datetime
import sys
import os
import logging

logger = logging.getLogger(__name__)


def progress_bar(iteration, total, length=50, prefix='', suffix='', fill='█'):
    """
    prints a inline progress bar into the console
    """
    percent = ("{0:.1f}").format(100 * (iteration / float(total)))
    filled_length = int(length * iteration // total)
    bar = fill * filled_length + '-' * (length - filled_length)
    sys.stdout.write(f'\r{prefix} |{bar}| {percent}% {suffix}')
    sys.stdout.flush()
    if iteration == total: 
        sys.stdout.write('\n')

def update_progress_bar(uploaded_size, total_size, unit, rounded_total_size):
    rounded_uploaded_size = 0
    if total_size > 1000000000:
        rounded_uploaded_size = round(uploaded_size / 1000000000, 2)
    else:
        rounded_uploaded_size = round(uploaded_size / 1000000, 2)
        
    suffix = f"{rounded_uploaded_size}{unit}/{rounded_total_size}{unit}"
    progress_bar(iteration=uploaded_size, total=total_size, prefix="Uploading", suffix=suffix)

def get_size_sum_files(files):
    """
    returns the total sum of the file sizes
    """
    sum = 0
    for f in files:
        try:
            sum += os.path.getsize(f)
        except FileNotFoundError as err:
            logger.error(err)
    return sum
        
def get_file_size(file):
    """
    returns the file size
    """
    try:
        return os.path.getsize(file)
    except FileNotFoundError as err:
        logger.error(err)
    return 0 

def exists_folder(folder_path, root):
    folders = folder_path.split("/")
    folders.remove("")

    current_folder = root.get_subfolder(folders[0])
    if current_folder is None:
        return False

    for index in range(1, len(folders)):
        current_folder = current_folder.get_subfolder(folders[index])
        if current_folder is None:
            return False
        
    return True

def convert_to_unix(date_str):
    """
    converts a date string with the format "%Y:%m:%d %H:%M:%S" into a unix timestamp
    """
    date_obj = datetime.datetime.strptime(date_str, "%Y:%m:%d %H:%M:%S")
    return int(date_obj.timestamp())

def build_url(base, extensions):
    """builds a valid url based on the specified base url and the extensions"""
    current_url = base
    for extension in extensions:
        if extension[0] == "/":
            current_url += extension
        else:
            current_url += f"/{extension}"
    return current_url