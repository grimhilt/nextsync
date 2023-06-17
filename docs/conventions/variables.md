# Conventions

## Path Variables

Considering cloning:
* ``https://nextcloud.example.com/remote.php/dav/files/grimhilt/dir/dir_to_clone``

We have (in ``ApiProps`` for example): 
* ``host``: ``https://nextcloud.example.com``
* ``username``: ``grimhilt``
* ``root``: ``/dir/dir_to_clone``

Concerning paths we have:
* ``remote_p``: ``/remote.php/dav/files/grimhilt/dir/dir_to_clone/D1/D1_F1.md``
* ``ref_p``: ``/home/grimhilt/dir_cloned``
* ``relative_p``: ``D1/D1_F1.md``
* ``abs_p``: ``/home/grimhilt/dir_cloned/D1_D1_F1.md``

Use prefix:
* ``p`` for Path or PathBuffer
* ``ps`` for String
