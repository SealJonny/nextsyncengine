## NextSyncEngine

### How it works
This CLI tool is meant to upload files from a single directory directly to Nextcloud. It will create a folder structure like:

- root
    - 2024
        - 1
            - 1
            - 2
            - 3
            - ...
        - 2
            - 1
            - 2
            - 3
            - ...
        - ...
    - 2023
        - 1
            - 1
            - 2
            - 3
            - ...
        - 2
            - 1
            - 2
            - 3
            - ...
        - ...
    - ...

or build on an existing one.
The folder structure will be cached to minimize the request to your Nextcloud which improves the speed and prevents overloading the server. 
**Please, do NOT change or delete the root folder and its content while the application is running!**

Any erros or warnings which might occure during the execution, will be logged and can be viewed in 'process.log'.

### Credentials
Place a '.env' file in the root directory of this application, copy and paste this code and replace it with your url and credentials (it is possible to use a Nextcloud App Password instead of your password).
```
NC_USERNAME=nextcloud_username
PASSWORD=password_OR_apppassword
SERVER_URL=https://nextcloud.example.com
EXIFTOOL=/path/to/exiftool/binary
```

### Usage
#### Python
Create a Python virtual environment and activate it.
##### Windows
```
.\venv\Scripts\activate
```

##### Linux
```
source venv/bin/activate
```

Now install the necessary modules with the requirements.txt.
```
python3 -m pip install -r requirements.txt
```

Now you should be able to run the code
```
python3 main.py --local_path /path/to/your/local/folder --remote_path /path/to/your/remote/folder
```

#### Binary
Place the binary in the location of your choice and create in the same folder a folder with the name _nextsyncengine_
```
cd path/to/binary
mkdir _nextsyncengine_
```

If you added the folder where the binary lies to PATH, you can use the CLI like this (you need sudo rights if the folder _nextsyncengine_ is owned by root)
```
sudo nextsyncengine --local_path /path/to/your/local/folder --remote_path /path/to/your/remote/folder
```