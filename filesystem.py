import os

class Folder:
    def __init__(self, name) -> None:
        self.name = name
        self.items = []
    
    def add_item(self, item, path_parent):
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
        subfolder.add_item(item, path_parent)
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
            
    def has_subfolder(self, path):
        """
        returns recursively whether a subfolder exists or not in a folder or its directory tree
        """
        # removes leading '/' if present
        if len(path) > 0 and path[0] == "/":
            path = path[1:len(path)]

        # the path could be fully walked which means the subfolder exists
        if path == "":
            return True

        # extract the next subfolder from path
        folder_names = path.split("/")
        subfolder_name = folder_names.pop(0)
        
        # get the next subfolder and if it returns 'None', it means that the path could not be walked fully. Therefore the subfolder does not exist.
        subfolder = self.get_subfolder(subfolder_name)
        if subfolder is None:
            return False
        
        # reasembles the path without the next subfolder
        path = ""
        for item in folder_names:
            path = os.path.join(path, item)

        # recursive function call for the subfolder
        return subfolder.has_subfolder(path)
            
    def to_string(self):
        """
        convertes the folder and its directory tree into a string
        """
        result = f"- {self.name}\n"
        for item in self.items:
            result += f"\t- {item.name}\n"
        return result

