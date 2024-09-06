# 🚀 NextSyncEngine

### 📖 Overview

NextSyncEngine is a powerful Rust-based CLI tool designed to streamline file uploads from a local directory directly to your Nextcloud instance. The tool automatically organizes files into a structured folder hierarchy, such as:

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


### 🚀 Features

- **Directory-Based Upload**: Seamlessly upload files from a local directory to Nextcloud while preserving a logical folder hierarchy. The tool automatically sorts files into nested folders organized by year, month, and day.

- **Unsorted Upload Option**: Alternatively, upload files in an unsorted manner, where files are uploaded directly to the specified folder without any directory restructuring.

- **Fall Back Upload**: In the event that one or more files fail to upload, you will be prompted to retry the upload process for the affected files. Should a critical error occur during the upload, the batch process will terminate, and the local paths of any remaining files will be recorded in a log file: `~/nextsyncengine_failed-uploads.txt` on Linux or `C:\Users\{username}\nextsyncengine_failed-uploads.txt` on Windows. This log can be used to retry uploads at a later time, for example, when the server is no longer in maintenance mode (have a look at the Usage doc).

- **Error Logging**: Detailed logging of any errors or warnings during execution is available in `process.log`, making it easier to troubleshoot issues.

### 🔐 Credentials & Settings
The credential and settings  are stored in a `.env` file. Replace the placeholders with your values.

```plaintext
NC_USERNAME=your_nextcloud_username
PASSWORD=your_password_or_apppassword
SERVER_URL=https://nextcloud.example.com
EXIFTOOL=/path/to/exiftool/binary
```

### 🔧 Installation
Either download the pre compiled binary from the latest release or compile the binary yourself:
```bash
git clone git@github.com:SealJonny/nextsyncengine.git
cd nextsyncengine
cargo build --release
```
You'll find the compiled binary at `path/to/nextsyncengine/target/release/nextsyncengine`.

Now move or copy the binary `nextsyncengine` to your desired location.
Create a folder named `_nextsyncengine_` in the same directory and place your `.env` file in `_nextsyncengine_`:
```bash
cd path/to/binary
mkdir _nextsyncengine_
cd _nextsyncengine_
mv /path/to/.env .
```

### ⚙️ Commands
**upload:sorted**
|Argument     |Option                           |Usage                                                                                                |Default Value    |
|:---         |:---                             |:---                                                                                                 |:---             |
|local_path   |-l\|--local_path &lt;local_path&gt;    |Path to a local folder containing the files you want to upload.                                      |no default value |
|file         |-f\|--file &lt;threads&gt;             |Path to the text file generated by nextsyncengine or any other text file with the same format.       |no default value |
|remote_path  |-r\|--remote_path <remote_path>  |Path to the location on Nextcloud where your files will be uploaded too.                             |no default value |
|depth        |-d\|--depth <depth>              |Let's you control the depth of the remote folder structure. Options are: year, month and day.        |month            |
|threads      |-t\|--threads <threads>          |Let's you control the number of threads used to upload the files. The value must be between 1 and 6. |3                |

**upload:unsorted**
|Argument     |Option                           |Usage                                                                                                |Default Value    |
|:---         |:---                             |:---                                                                                                 |:---             |
|local_path   |-l\|--local_path <local_path>    |Path to a local folder containing the files you want to upload.                                      |no default value |
|file         |-f\|--file <file>                |Path to the text file generated by nextsyncengine or any other text file with the same format.       |no default value |
|remote_path  |-r\|--remote_path <remote_path>  |Path to the location on Nextcloud where your files will be uploaded too.                             |no default value |
|threads      |-t\|--threads <threads>          |Let's you control the number of threads used to upload the files. The value must be between 1 and 6. |3                |

 **⚠️Important: Do NOT change or delete the local and remote root folder or their content while the application is running!**


### 📜 Logs
In case of a failure, check `process.log` for any errors or warnings that occured during execution.
Path: `/path/to/_nextsyncengine_/process.log`