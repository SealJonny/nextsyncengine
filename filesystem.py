import os

class Folder:
    def __init__(self, name) -> None:
        self.name = name
        self.items = []
    
    def add_item(self, item, path_parent):
        """
        adds a folder in its parent directory
        """
        # remove starting folder from path_parent
        path_parent = path_parent.removeprefix(self.name)
        return self.__add_item_rec(item, path_parent)

    def __add_item_rec(self, item, path_parent):
        # removes leading '/' if present
        if len(path_parent) > 0 and path_parent[0] == "/":
            path_parent = path_parent[1:len(path_parent)]
        
        # extract the subfolder from path
        folder_names = path_parent.split("/")
        subfolder_name = folder_names.pop(0)

        # get the next subfolder which should be a parent to the new folder. If it returns 'None', it means that the current folder is the parent
        # and the folder will be added to its items.
        subfolder = self.get_subfolder(subfolder_name)
        if subfolder is None:
            self.items.append(item)
            return
        
        # reasembles the path without the next subfolder
        path_parent = ""
        for name in folder_names:
            path_parent = os.path.join(path_parent, name)
        
        # recursive function call for the next subfolder
        subfolder.__add_item_rec(item, path_parent)
        return

    def remove_item(self, item_name):
        for index in range(len(self.items)):
            if self.items[index].name == item_name:
                self.items.pop(index)
                break

    def get_subfolder(self, name):
        for index in range(len(self.items)):
            if self.items[index].name == name:
                return self.items[index]
        return None
            
    def has_subfolder(self, path_folder):
        """
        returns recursively whether a subfolder exists or not in a folder or its directory tree
        """
        # remove starting folder from path_parent
        path_folder = path_folder.removeprefix(self.name)
        return self.__has_subfolder(path_folder)

    def __has_subfolder(self, path_folder):
        # removes leading '/' if present
        if len(path_folder) > 0 and path_folder[0] == "/":
            path_folder = path_folder[1:len(path_folder)]

        # the path could be fully walked which means the subfolder exists
        if path_folder == "":
            return True

        # extract the next subfolder from path
        folder_names = path_folder.split("/")
        subfolder_name = folder_names.pop(0)
        
        # get the next subfolder and if it returns 'None', it means that the path could not be walked fully. Therefore the subfolder does not exist.
        subfolder = self.get_subfolder(subfolder_name)
        if subfolder is None:
            return False
        
        # reasembles the path without the next subfolder
        path_folder = ""
        for item in folder_names:
            path_folder = os.path.join(path_folder, item)

        # recursive function call for the subfolder
        return subfolder.__has_subfolder(path_folder)
            
    def to_string(self, depth=0):
        """
        convertes the folder and its directory tree into a string
        """
        result = ""
        indention = ""
        for i in range(depth):
            indention += "\t"
        result = f"{indention}- {self.name}\n"

        if len(self.items) == 0:
            return result

        indention += "\t"
        for item in self.items:
            next_result = item.to_string(depth + 1)
            if next_result != f"{depth + 1}- item.name":
                result += next_result
            else:
                result += f"{indention}- {item.name}"

        return result

