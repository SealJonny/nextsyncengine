import os
import helpers
from nextcloud_api import Nextcloud_Client
from dotenv import load_dotenv
from collections import deque
from filesystem import Folder
import logging_config
import logging
from requests import exceptions
import media
import time
import datetime


load_dotenv()
server_url = os.getenv("SERVER_URL")
username = os.getenv("NC_USERNAME")
password = os.getenv("PASSWORD")

client = Nextcloud_Client(server_url, username, password)
logger = logging.getLogger(__name__)

def travel_dir(root_dir):
    """
    returns a list of absolut paths to all files in a directory and its subdirectories
    """
    paths_dir = deque()
    paths_dir.appendleft(root_dir)

    paths_file = deque()

    while len(paths_dir) != 0:
        current_dir = paths_dir.popleft()
        for item in os.listdir(current_dir):
            path_item = os.path.join(current_dir, item)
            if os.path.isdir(path_item):
                paths_dir.appendleft(path_item)
            else:
                paths_file.appendleft(path_item)
    return paths_file

def travel_dir_dav(root_dir):
    """
    travels the specified directory and returns its structure
    """
    root = Folder(root_dir)

    paths_dir = deque()
    paths_dir.appendleft(root_dir)

    # goes through all subfolders and inserts them into the structure
    while len(paths_dir) != 0:
        current_dir = paths_dir.popleft()

        # list the current remote directory
        results, err = client.ls(current_dir)
        if err is not None:
            if isinstance(err, exceptions.HTTPError):
                if err.response.status_code == 404:
                    logger.warning(err)
                else:
                    logger.error(err)
            else:
                logger.error(err)
        else:
            # append the subfolders to the queue and insert them into the directory structure
            for item in results.items():
                if item[1] is not False:
                    paths_dir.appendleft(os.path.join(current_dir, item[0]))
                    root.add_item(Folder(item[0]), current_dir)
    return root


def get_time_subfolder(dst_root, file):
    """
    returns based on the timestamp of the file its destination folder and creates them if necessacery
    """
    _, mtime_unix = media.get_datetime(file)

    if mtime_unix is None:
        logger.warning("No timestamps found: File will be uploaded into root directory")
        return dst_root.name
    # ctime = time.ctime(ctime)
    # mtime = time.ctime(mtime)

    mtime = datetime.datetime.fromtimestamp(mtime_unix)
    year_str = str(mtime.year)
    month_str = str(mtime.month)
    day_str = str(mtime.day)

    # check if the year folder exists and create it if not
    year_path = os.path.join(dst_root.name, year_str)
    if not dst_root.has_subfolder(year_str):
        client.create_folder(year_path)
        dst_root.add_item(Folder(year_str), dst_root.name)

    # check if the month folder exists and create it if not
    month_path = os.path.join(year_path, month_str)
    if not dst_root.has_subfolder(month_path):
        client.create_folder(month_path)
        dst_root.add_item(Folder(month_str), year_path)

    # check if the day folder exists and create it if not
    day_path = os.path.join(month_path, day_str)
    if not dst_root.has_subfolder(day_path):
        client.create_folder(day_path)
        dst_root.add_item(Folder(day_str), month_path)

    return day_path
    

def upload_folder(files, root):
    """
    uploads a local folder to a specified destination on Nextcloud
    """
    for file in files:
        # get the destination of the file and upload it
        dst = get_time_subfolder(root, file)
        err = client.upload_file(file, os.path.join(dst, os.path.basename(file)))
        if err is not None:
            logger.error(err)
        else:
            print(f"successfully upload {file} to nextcloud")
def main():
    # folder_src = ""
    # while os.path.isdir(folder_src) == False:
    #     folder_src = input("Enter the path of the file you want to upload: ")
    # folder_dst = input("Enter the location where the file should be saved: ")

    # files = travel_dir(folder_src)

    dir_dav = "/Test"
    if not client.exists_folder("/Test")[0]:
        return
    
    dir_local = "/home/sealjonny/Downloads/Converted"
    files = travel_dir(dir_local)
    root = travel_dir_dav(dir_dav)

    print(root.to_string())
    upload_folder(files, root)
    print(root.to_string())




main()