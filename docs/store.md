
## Blob object

```
file_name timestamp1 size timestamp2 hash
```
timestamp1: timestamp of file on server to know if the server has an update
timestamp2: timestamp of file locally to know when the file has changed on the system

## Tree object
```
folder_name timestamp
tree hash_path folder_name
blob hash_path file_name
```