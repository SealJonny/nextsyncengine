## NextSyncEngine

### How it works
This application is meant to upload files from a single directory directly to Nextcloud. It will create a folder structure like:

- root
    - 2024
        - 1
            -1
            -2
            -3
            ...
        -2
            -1
            -2
            -3
            ...
        ...
    -2023
        -1
            -1
            -2
            -3
            ...
        -2
            -1
            -2
            -3
            ...
        ...
    ...

or build on an existing one.
The folder structure will be cached to minimize the request to your Nextcloud which improves the speed and prevents overloading the server. 
**Please, do NOT change or delete the root folder and its content while the application is running!**

Any erros or warning which might occure during the execution, will be logged and can be viewed in 'process.log'.
