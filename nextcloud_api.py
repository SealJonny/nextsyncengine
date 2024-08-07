import requests
import requests.auth
import os
import helpers
from bs4 import BeautifulSoup

class Nextcloud_Client:
    def __init__(self, url_server, username, password) -> None:
        self.__url_dav = helpers.build_url(url_server, ["remote.php/dav/files", username])
        self.__auth = requests.auth.HTTPBasicAuth(username, password)

    def upload_file(self, path_src, path_dst, use_time=True):
        """
        uploads a file to the specified location
        (use_time = True => transfers local unix timestamp to the uploaded file
        use_time = False => uses time of the upload as the timestamp)
        """
        # mtime = modified, ctime = creation
        if use_time:
            headers = {
                "X-OC-MTime": f"{int(os.path.getmtime(path_src))}",
                "X-OC-CTime": f"{int(os.path.getctime(path_src))}"
            }
        else:
            headers = {}
        
        url = helpers.build_url(self.__url_dav, [path_dst])

        response = requests.put(url=url, headers=headers, auth=self.__auth, data=open(path_src, 'rb'))
        if response.status_code != 201 or response.status_code != 204:
            print(f"uploading the file {path_src} failed with the status code {response.status_code}:\n{response.text}")

    def __extract_displayname(self, xml):
        """extracts the displaynames and whether the item is a file or not and returns it as a dictionary"""
        
        soup = BeautifulSoup(xml, 'xml')

        items = {}
        for item in soup.find_all("d:prop"):
            displayname = item.find("d:displayname").text
            resourcetype = item.find("d:resourcetype").find("d:collection")
            items[displayname] = False if resourcetype is None else True
        return items

    def ls(self, dir):
        """
        returns a list of the content of the specified directory
        """

        headers = {
            'Content-Type': 'application/xml',
            'Depth': '1'
        }

        data = """<?xml version='1.0'?> 
                    <d:propfind xmlns:d="DAV:" >
                        <d:prop>
                            <d:displayname />
                            <d:resourcetype />
                        </d:prop>
                    </d:propfind>
        """

        url = helpers.build_url(self.__url_dav, [dir]
                                )
        response = requests.request(method="PROPFIND", url=url, headers=headers, auth=self.__auth, data=data)
        if response.status_code != 207:
            print(f"listing the dav directory {dir} failed with status {response.status_code}:\n{response.text}")
            return None
        
        # extract file names and information wether its a file or folder from the response and drop the directory which is being listed
        items = self.__extract_displayname(response.text)
        if dir == "/":
            items.pop(self.__auth.username)
        else:
            items.pop(os.path.basename(dir))
        
        return items

    
    def request(self, method, dav_path, headers=None, data=None):
        """
        open nextcloud api endpoint for making customized requests to your nextcloud server 
        """

        url = helpers.build_url(self.__url_dav, dav_path)
        return requests.request(method=method, url=url, headers=headers, auth=self.__auth, data=data)