import os
import helpers
from nextcloud_api import Nextcloud_Client
from dotenv import load_dotenv

load_dotenv()

server_url = os.getenv("SERVER_URL")
username = os.getenv("NC_USERNAME")
password = os.getenv("PASSWORD")

client = Nextcloud_Client(server_url, username, password)

while os.path.isdir(folder_src) == False:
    folder_src = input("Enter the path of the file you want to upload: ")
folder_dst = input("Enter the location where the file should be saved: ")

files = helpers.travel_dir(folder_src)