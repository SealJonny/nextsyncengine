import os
from nextcloud_api import Nextcloud_Client
from dotenv import load_dotenv
from collections import deque
from filesystem import Folder, File
import logging_config
import logging
from requests import exceptions
from media import Extract
import datetime
import argparse
import sys
import helpers


# check if it is executed as a binary or script and set paths accordingly
if hasattr(sys, 'frozen'):
    binary_path = os.path.dirname(sys.executable)
    load_dotenv(os.path.join(binary_path, "_nextsyncengine_", ".env"))
    logging_config.configure_logger(binary_path)
else:
    load_dotenv()
    logging_config.configure_logger("")

server_url = os.getenv("SERVER_URL")
username = os.getenv("NC_USERNAME")
password = os.getenv("PASSWORD")
exiftool = os.getenv("EXIFTOOL")


client = Nextcloud_Client(server_url, username, password)
extractor = Extract()
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


def get_time_folder(dst_root, file, depth):
    """
    returns the ctime and mtime timestamps and based on the mtime it returns the destination of the file (and creates the destination folder if necessesary)
    """
    ctime_unix, mtime_unix = extractor.get_datetime(file)

    if mtime_unix is None:
        logger.warning("No timestamps found: File will be uploaded into root directory")
        return dst_root.name, None, None

    mtime = datetime.datetime.fromtimestamp(mtime_unix)
    year_str = str(mtime.year)
    if mtime.month < 10:
        month_str = f"0{mtime.month}"
    else:
        month_str = str(mtime.month)

    if mtime.day < 10:
        day_str = f"0{mtime.day}"
    else:
        day_str = str(mtime.day)

    # check if the year folder exists and create it if not
    year_path = os.path.join(dst_root.name, year_str)
    if not dst_root.has_subfolder(year_str):
        err = client.create_folder(year_path)
        if err is not None:
            logger.error(err)
            return dst_root.name, ctime_unix, mtime_unix
        
        dst_root.add_item(Folder(year_str), dst_root.name)
    
    if depth == "year":
        return year_path, ctime_unix, mtime_unix

    # check if the month folder exists and create it if not
    month_path = os.path.join(year_path, month_str)
    if not dst_root.has_subfolder(month_path):
        err = client.create_folder(month_path)
        if err is not None:
            logger.error(err)
            return dst_root.name, ctime_unix, mtime_unix
        
        dst_root.add_item(Folder(month_str), year_path)
    
    if depth == "month":
        return month_path, ctime_unix, mtime_unix

    # check if the day folder exists and create it if not
    day_path = os.path.join(month_path, day_str)
    if not dst_root.has_subfolder(day_path):
        err = client.create_folder(day_path)
        if err is not None:
            logger.error(err)
            return dst_root.name, ctime_unix, mtime_unix
        
        dst_root.add_item(Folder(day_str), month_path)

    return day_path, ctime_unix, mtime_unix
    

def upload_folder(local_paths, root, depth):
    """
    uploads a local folder to a specified destination on Nextcloud
    """
    sum_size = helpers.get_size_sum_files(local_paths)
    uploaded_size = 0
    rounded_total_size = 0
    unit = ""

    GB = 1000000000
    MB = 1000000 
    if sum_size > GB:
        rounded_total_size = round(sum_size / GB, 2)
        unit = "G"
    else:
        rounded_total_size = round(sum_size / MB, 2)
        unit = "M"

    helpers.update_progress_bar(0, sum_size, unit, rounded_total_size)
    
    for index in range(len(local_paths)):
        local_path = local_paths[index]

        # get the destination of the file and upload it
        file_size = helpers.get_file_size(local_path)
        remote_parent, ctime, mtime = get_time_folder(root, local_path, depth)

        file = File(local_path=local_path, remote_parent=remote_parent, ctime=ctime, mtime=mtime, size=file_size)

        last_upload = False
        if len(local_paths) - index == 1:
            last_upload = True

        err = client.upload_file(file, last_upload)
        if err is not None:
            logger.error(err)
            continue
        
        uploaded_size += file_size
        helpers.update_progress_bar(uploaded_size, sum_size, unit, rounded_total_size)

def main():
    # check if all necessary values exist in the .env file and terminate execution if not
    if username is None:
        logger.error(NameError("No environment variable with the name 'NC_USERNAME' found!"))
        print("Check the environment variable 'NC_USERNAME'!")
        return
    
    if password is None:
        logger.error(NameError("No environment variable with the name 'PASSWORD' found!"))
        print("Check the environment variable 'PASSWORD'!")
        return

    if server_url is None:
        logger.error(NameError("No environment variable with the name 'SERVER_URL' found!"))
        print("Check the environment variable 'SERVER_URL'!")
        return
    
    if exiftool is None:
        logger.error(NameError("No environment variable with the name 'EXIFTOOL' found!"))
        print("Exiftool could not be located!")
        return

    print("Checking if Nextcloud server is online ...")
    is_online, err = client.is_online()
    if err:
        logger.error(err)

    if not is_online:
        print("The Nextcloud server is in maintenance mode. Please, try again later!")
        return
    print("done")
    
    # add local_path and remote_path as inline options
    parser = argparse.ArgumentParser(description="Process a path argument")
    parser.add_argument("--local_path", type=str, required=True, help="The path to your local folder which will be uploaded")
    parser.add_argument("--remote_path", type=str, required=True, help="The location where on you Nextcloud server the folder will be uploaded")
    parser.add_argument("--depth", type=str, required=False, default="month", help="Options are: year, month and day. Determines the depth of the folder structure.")
    args = parser.parse_args()
    
    local_path = args.local_path.strip()
    remote_path = args.remote_path.strip().rstrip("/")
    depth = args.depth.strip().lower()

    # check if depth was specified correct
    if depth != "year" and depth != "month" and depth != "day":
        print(f"incorrect value {depth} for option --depth!")
        return
    
    # check if there is a folder at local_path
    if not os.path.isdir(local_path):
        print(f"The specified local folder {local_path} is not a directory!")
        return
    
    # check if there is a folder at remote_path
    exists, err = client.exists_folder(remote_path)
    if err is not None:
        logger.error(err)
        print("Checking if remote folder exists failed! Check process.log for more details.")
        return
    
    # ask user if he wants to create the remote folder if it does not exists
    if not exists:
        ans = input(f"The folder {remote_path} does not exist on your Nextcloud instance.\nDo you want to create it?\nYes(y) or No(n) ")
        if ans.lower().strip() == "y" or ans.lower().strip() == "yes":
            client.create_folder(remote_path)
        else:
            return
    
    # create a list of all files which will be uploaded
    local_paths = travel_dir(local_path)
    
    # cache remote folder structure with remote_path as root
    root = travel_dir_dav(remote_path)

    # set exiftool path and terminate execution if there is no file at 'exiftool'
    err = extractor.set_exiftool(exiftool)
    if err:
        logger.error(err)
        print("Exiftool could not be located!")
        return

    # upload all files to remote_path
    upload_folder(local_paths, root, depth)




main()