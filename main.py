import os
import helpers
from nextcloud_api import Nextcloud_Client
from dotenv import load_dotenv
from collections import deque
from filesystem import Folder
import logging_config
import logging
from requests import exceptions


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

def upload_folder(files, root, dst):
    """
    uploads a local folder to a specified destination on Nextcloud
    """
    # remove root from path, split it into the single folder names and reverse it
    non_root_path = dst.removeprefix(root.name)
    folders = non_root_path.split("/")
    try:
        folders.remove("")
    except ValueError as err:
        pass
    folders.reverse()

    missing_folders = deque()

    # iters through the folders and checks if they exits on the Nextcloud instance. If not they will be added to missing_folders
    while len(folders) != 0:
        path_folder = ""
        for folder in folders:
            path_folder = os.path.join(folder, path_folder)

        if not root.has_subfolder(path_folder):
            missing_folders.appendleft(path_folder)
            folders.pop(0)

    # iters through missing_folders and creates them
    for dir in missing_folders:
        abs_path = os.path.join(root.name, dir)

        # removing trailing '/' because it messes with the os.path.basename and .dirname functions
        if abs_path[len(abs_path) - 1] == "/":
            abs_path = abs_path[:len(abs_path) - 1]

        # creates the current folder and updated the directory structure
        logger.warning(f"Folder not found, {abs_path} does not exists in this Nextcloud instance")
        client.create_folder(abs_path)
        root.add_item(Folder(os.path.basename(abs_path)), os.path.dirname(abs_path))

    # uploading the files to their destination
    for file in files:
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
    dir_local = "/home/sealjonny/Downloads/Converted"
    files = travel_dir(dir_local)
    root = travel_dir_dav(dir_dav)

    print(root.to_string())
    upload_folder(files, root, f"{dir_dav}/Photos")
    print(root.to_string())




main()