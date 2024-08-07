import nextcloud_api
import helpers
from filesystem import Folder
from dotenv import load_dotenv
import os

load_dotenv(".env")

server_url = os.getenv("SERVER_URL")
username = os.getenv("NC_USERNAME")
password = os.getenv("PASSWORD")

client = nextcloud_api.Nextcloud_Client(server_url, username, password)
root = Folder("/")

results = client.ls("/")

print(results)

for result in results.items():
    if result[1] == True:
        root.add_item(Folder(result[0]))

results = client.ls("/Photos")

folder = root.get_subfolder("Photos")

for result in results.items():
    if result[1] == True:
        folder.add_item(Folder(result[0]))

print(folder.has_subfolder("/Neuer Ordner/test"))


