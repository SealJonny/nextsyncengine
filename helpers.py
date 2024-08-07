import datetime
import re
from collections import deque
import os

def get_subdictionary(path, dict):
    current_path, item = os.path.split(path)
    splitted_path =  deque()
    splitted_path.appendleft(item)

    while current_path != "/":
        current_path, item = os.path.split(current_path)
        splitted_path.appendleft(item)
    try:
        for key in splitted_path:
            dict = dict[key]
    except KeyError as err:
        return None
    return dict

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
        current_dict = get_subdictionary(current_dir)
        current_dir = tmp_dict
    return dir_structure

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
    date_obj = datetime.datetime.strptime(date_str, "%Y-%m-%d %H:%M:%S")
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