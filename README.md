# 🚀 NextSyncEngine

### 📖 Overview

NextSyncEngine is a powerful CLI tool designed to streamline file uploads from a local directory directly to your Nextcloud instance. The tool automatically organizes files into a structured folder hierarchy, such as:

- **root**
  - **2024**
    - **01**
      - 01
      - 02
      - 02
      - ...
    - **02**
      - 01
      - 02
      - 03
      - ...
    - ...
  - **2023**
    - **01**
      - 01
      - 02
      - 03
      - ...
    - **02**
      - 01
      - 02
      - 03
      - ...
    - ...
  - ...

🕒 Caching: The folder structure is cached to improve performance. 

⚠️ **Important: Do NOT change or delete the local and remote root folder or their content while the application is running!**


### 🔐 Credentials & Settings
The credential and settings  are stored in a `.env` file. Replace the placeholders with your values.

```plaintext
NC_USERNAME=your_nextcloud_username
PASSWORD=your_password_or_apppassword
SERVER_URL=https://nextcloud.example.com
EXIFTOOL=/path/to/exiftool/binary
```

### 🔧 Installation
#### 🛠️ Binary
Place the binary `nextsyncengine`in your desired location.
Create a folder named `_nextsyncengine_` in the same directory and place your `.env` file in `_nextsyncengine_`:
```bash
cd path/to/binary
mkdir _nextsyncengine_
cd _nextsyncengine_
mv /path/to/.env .
```

#### 🐍 Python
If you prefer running the source code, follow these steps:

First, create and activate a Python virtual environment.
##### 💻 Windows
 ```
.\venv\Scripts\activate
```

##### 🐧 Linux
```
source venv/bin/activate
```

Next, install the required Python packages using requirements.txt:
```bash
python3 -m pip install -r requirements.txt
```

Place the .env file into the the root folder of this application:
```bash
cd /path/to/root/folder
mv /path/to/.env .
```


### ⚙️ Usage
#### 🛠️ Binary
If you’ve added the binary’s location to your PATH, you can use the CLI like this (you may need to execute with sudo rights if `_nextsyncengine_` and its content is owned by root):
```bash
nextsyncengine --local_path /path/to/your/local/folder --remote_path /path/to/your/remote/folder --depth <Options: year, month(default), day>
```

#### 🐍 Python
```bash
python3 main.py --local_path /path/to/your/local/folder --remote_path /path/to/your/remote/folder --depth <Options: year, month(default), day>
```


### 📜 Logs
In case of a failure, check `process.log` for any errors or warnings that occured during execution.

#### 🛠️ Binary
Path: `/path/to/_nextsyncengine_/process.log`

#### 🐍 Python
Path: `/path/to/application/process.log`