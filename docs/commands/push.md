
## Push

1. Get list of file and folder to push from ``.nextsync/index``
2. For each object in the list:
    * If it is a file:
        * Check if the file exists on the server
        * If a collision occurs, compare the modification date (given by the file blob and the server)
            * If it is more recent download the file under with ``.dist`` extension
            * Else overwrite the file
        * If not collision occurs, upload the file
    * If is it a folder:
        * If the folder exists on the server apply this procedure for its content
        * Else upload it