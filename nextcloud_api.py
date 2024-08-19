import requests
import requests.auth
import os
import helpers
from bs4 import BeautifulSoup
import hashlib
import secrets
import string
import tempfile

class Nextcloud_Client:
    def __init__(self, url_server, username, password) -> None:
        self.__base_url = url_server
        self.__url_dav = helpers.build_url(url_server, ["remote.php/dav/files", username])
        self.session = requests.Session()
        self.session.auth = requests.auth.HTTPBasicAuth(username, password)
        self.bulk_files = []
        self.bulk_size = 0
    
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

    def __bulk_upload_files(self):
        def random_hex(length):
            return ''.join(secrets.choice(string.hexdigits.lower()) for _ in range(length))
        TMP_FOLDER = tempfile.gettempdir()
        UPLOAD_PATH = os.path.join(TMP_FOLDER, f"bulk_upload_request_{random_hex(8)}.txt")
        BOUNDARY = f"boundary_{random_hex(8)}"

        try:
            with open(UPLOAD_PATH, 'wb') as upload_file:
                for file in self.bulk_files:
                    file_remote_path = f"{file.remote_parent}/{os.path.basename(file.local_path)}"

                    with open(file.local_path, 'rb') as f:
                        file_hash = hashlib.md5(f.read()).hexdigest()

                    upload_file.write(f"--{BOUNDARY}\r\n".encode())
                    upload_file.write(f"X-File-Path: {file_remote_path}\r\n".encode())
                    upload_file.write(f"X-OC-Mtime: {file.mtime}\r\n".encode())
                    upload_file.write(f"X-File-Md5: {file_hash}\r\n".encode())
                    upload_file.write(f"Content-Length: {file.size}\r\n\r\n".encode())

                    with open(file.local_path, 'rb') as f:
                        upload_file.write(f.read())

                    upload_file.write("\r\n".encode())

                upload_file.write(f"--{BOUNDARY}--\r\n".encode())

            url_bulk = helpers.build_url(self.__base_url, ["remote.php/dav/bulk"])
            with open(UPLOAD_PATH, 'rb') as f:
                response = self.session.post(
                    url=url_bulk,
                    headers={"Content-Type": f"multipart/related; boundary={BOUNDARY}"},
                    data=f
                )
            
            response.raise_for_status()
        except (FileNotFoundError, requests.RequestException) as err:
            return err
        finally:
            if os.path.exists(UPLOAD_PATH):
                os.remove(UPLOAD_PATH)
        return None
    
    def __normal_upload_file(self, file):
        """
        uploads a file to the specified location
        """
        # mtime = modified, ctime = creation
        headers = {
            "X-OC-MTime": f"{file.mtime}",
            "X-OC-CTime": f"{file.ctime}"
        }

        url = helpers.build_url(self.__url_dav, [file.remote_parent, os.path.basename(file.local_path)])
        try:
            with open(file.local_path, 'rb') as file:
                response = self.session.put(url=url, headers=headers, data=file)
                response.raise_for_status()
        except (requests.RequestException, FileNotFoundError) as err:
            return err
        return None

    def upload_file(self, file, last_upload):
        bulk_uploaded = False
        if file.size <= 5000000:
            self.bulk_files.append(file)
            self.bulk_size += file.size
            bulk_uploaded = True
        
        err = None
        if self.bulk_size >= 75000000 or last_upload:
            err = self.__bulk_upload_files()
            self.bulk_size = 0
            self.bulk_files = []

        if not bulk_uploaded:
            err = self.__normal_upload_file(file)
        return err

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