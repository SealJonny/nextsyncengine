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