import os
import helpers
from nextcloud_api import Nextcloud_Client
from dotenv import load_dotenv
from collections import deque

# returns a list of absolut paths to all files in a directory and its subdirectories
def travel_dir(root_dir):
    #paths_dir = deque(os.path.join(root_dir + obj) for obj in os.listdir(root_dir) if os.path.isdir(os.path.join(root_dir +obj)))
    #paths_file = deque(os.path.join(root_dir + obj) for obj in os.listdir(root_dir) if not os.path.isdir(os.path.join(root_dir +obj)))
    
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

def travel_dir_dav(root_dir, nextcloud_client):
    """
    Traversiert das Verzeichnis und bildet die Struktur in einem Dictionary ab.
    """
    dir_structure = {}
    paths_dir = deque()
    paths_dir.appendleft(root_dir)

    while len(paths_dir) != 0:
        current_dir = paths_dir.popleft()
        tmp_dict = {}
        for item in nextcloud_client.ls(current_dir).items():
            if item[1] is not None:
                paths_dir.appendleft(os.path.join(current_dir, item[0]))
                tmp_dict[item[0]] = {}
        current_dict = helpers.get_subdictionary(current_dir)
        current_dir = tmp_dict
    return dir_structure

def main():
    load_dotenv()
    server_url = os.getenv("SERVER_URL")
    username = os.getenv("NC_USERNAME")
    password = os.getenv("PASSWORD")

    client = Nextcloud_Client(server_url, username, password)

    while os.path.isdir(folder_src) == False:
        folder_src = input("Enter the path of the file you want to upload: ")
    folder_dst = input("Enter the location where the file should be saved: ")

    files = helpers.travel_dir(folder_src)


main()