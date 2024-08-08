import nextcloud_api
import helpers
from filesystem import Folder
from dotenv import load_dotenv
import os
import logging_config
import logging

from requests import exceptions

load_dotenv(".env")

server_url = os.getenv("SERVER_URL")
username = os.getenv("NC_USERNAME")
password = os.getenv("PASSWORD")

client = nextcloud_api.Nextcloud_Client(server_url, username, password)
logger = logging.getLogger(__name__)

result, err = client.ls("/Photo")

if err is not None:
    if isinstance(err, exceptions.HTTPError):
        if err.response.status_code == 404:
            logger.warning(err)
    else:
        logger.error(err)

exists, err = client.exits_folder("/Photo")

if err is not None:
    logger.error(err)

print(exists)

# print(err.)
# print(result)

# client.upload_file("/home/sealjonny/test.txt", "/Photos/Robin_Stinkt.txt")
# root = Folder("/")

# root.add_item(Folder("Photos"), "/")
# root.add_item(Folder("Test"), "/Photos")
# root.add_item(Folder("Notizen"), "/Photos")

# print(root.has_subfolder("/"))
# folder = root.get_subfolder("Photos")
# print(folder.to_string())

# results = client.ls("/")

# print(results)

# for result in results.items():
#     if result[1] == True:
#         root.add_item(Folder(result[0]))

# results = client.ls("/Photos")

# folder = root.get_subfolder("Photos")

# for result in results.items():
#     if result[1] == True:
#         folder.add_item(Folder(result[0]))

# print(folder.has_subfolder("/Neuer Ordner/test"))


