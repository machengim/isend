Target usage
---

Sender side:

```
ishare -s -e 10 -l 6 -p 12345 file1 dir1 -m "Hello receiver!" --shutdown
```

Receiver side:

```
ishare -r -e 10 -p 54321 -d ./receive a0178f --shutdown --ovewrite
```

version 1

```
ishare -s file1 dir1
```

```
ishare -r a0178f
```

Process:

1. Machine A check files and directories, listen on a UDP port, and generate a code made up of the port number and a random code. So the first 4 characters are hexademical expression of the port number, and the rests are a random password. Assume the code is `a0178f`.

2. Input the code on machine B which will start listening on a TCP port and send a broadcast to local network devices with port `a017` with the code from the TCP port on machine B and another random password. Assume this code is `05df79` and it will be received by machine A.

3. Machine A use this code `05df79` to connect to machine B's TCP port, and start sending files.