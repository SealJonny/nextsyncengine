import requests
import requests.auth
import os
import helpers
from bs4 import BeautifulSoup

class Nextcloud_Client:
    def __init__(self, url_server, username, password) -> None:
        self.__base_url = url_server
        self.__url_dav = helpers.build_url(url_server, ["remote.php/dav/files", username])
        self.__auth = requests.auth.HTTPBasicAuth(username, password)
        self.session = requests.Session()
        self.session.auth = self.__auth
    
    def is_online(self):
        """
        returns False if server is in maintenance mode
        """
        response = self.session.get(url=self.__base_url)
        if response.status_code != 503 and response.status_code != 500:
            return True, None
        soup = BeautifulSoup(response.content, 'html.parser')
        
        # find 'h2' and check its content to determine if maintenance mode is enabled or not
        maintenance_header = soup.find('h2')
        if maintenance_header:
            if maintenance_header.text.strip() == "Maintenance mode":
                return False, None

        try: 
            response.raise_for_status()
        except requests.RequestException as err:
            return False, err

    def upload_file(self, path_src, path_dst, ctime="", mtime="", use_time=True):
        """
        uploads a file to the specified location
        (use_time = True => transfers local unix timestamp to the uploaded file
        use_time = False => uses time of the upload as the timestamp)
        """
        # mtime = modified, ctime = creation
        if use_time:
            headers = {
                "X-OC-MTime": mtime,
                "X-OC-CTime": ctime
            }
        else:
            headers = {}
        
        url = helpers.build_url(self.__url_dav, [path_dst])
        try:
            with open(path_src, 'rb') as file:
                response = self.session.put(url=url, headers=headers, data=file)
                response.raise_for_status()
        except (requests.RequestException, FileNotFoundError) as err:
            return err
        return None


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

        url = helpers.build_url(self.__url_dav, [dir])

        # proping the folder 'dir' via http request
        try:
            response = self.session.request(method="PROPFIND", url=url, headers=headers, data=data)
            response.raise_for_status()
        except requests.RequestException as err:
            return None, err

        # extract file names and information wether its a file or folder from the response and drop the directory which is being listed
        items = self.__extract_displayname(response.text)
        if dir == "/":
            items.pop(self.__auth.username)
        else:
            items.pop(os.path.basename(dir))
        
        return items, None
    

    def exists_folder(self, dir):
        """
        returns whether a folder exists in a Nextcloud instance, 'dir' must be the full path to the folder.
        """

        # proping the folder 'dir' and determining based on the response and its status code if this folder exits or not
        result, err = self.ls(dir)
        if err is not None:
            if isinstance(err, requests.exceptions.HTTPError):
                if err.response.status_code == 404:
                    return False, None
                else:
                    return False, err
            else:
                return False, err
        return True, None

    def create_folder(self, dir):
        url = helpers.build_url(self.__url_dav, [dir])
        try:
            response = self.session.request(method="MKCOL", url=url)
            response.raise_for_status()
        except requests.RequestException as err:
            return err
        return None
    
    def request(self, method, dav_path, headers=None, data=None):
        """
        open nextcloud api endpoint for making customized requests to your nextcloud server 
        """

        url = helpers.build_url(self.__url_dav, dav_path)
        
        try:
            response = self.session.request(method=method, url=url, headers=headers, data=data)
            response.raise_for_status()
        except requests.RequestException as err:
            return response, err
        
        return response, None