import os

class Folder:
    def __init__(self, name) -> None:
        self.name = name
        self.items = []
    
    def add_item(self, item):
        self.items.append(item)

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
        if path[0] == "/":
            path = path[1:len(path)]

        if path == "":
            return True
        
        if path == "/":
            return False 

        folder_names = path.split("/")
        if len(folder_names) == 0:
            return False

        subfolder_name = folder_names.pop(0)

        path = ""
        for item in folder_names:
            path = os.path.join(path, item)
        
        subfolder = self.get_subfolder(subfolder_name)

        if subfolder is None:
            return False
        
        return subfolder.has_subfolder(path)
            
    def to_string(self):
        """
        convertes the folder and its directory tree into a string
        """
        result = f"- {self.name}\n"
        for item in self.items:
            result += f"\t- {item.name}\n"
        return result

