# ioc

## flow

### check destination
This check depends on the checksum file to be saved in a different location!

1. read hash from file
2. calculate the current checksum
3. compare against stored value


### build
1. find source directory
1. copy source to staging directory
1. do the startup wrapping via a template
2. calculate the checksum and write to the destination


## ioc install

## TODO

- md5
- template wrapper
- 