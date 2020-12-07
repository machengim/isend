Isend workflow
---

1. Sender parses command arguments as below:

`isend -s -p -e 10 -m Hello a.txt b.png ~/Documents/`

   + If `-p` is specified, a new line will prompt to ask for the password.

   + If `-e` is not specified, the expire time is set to default 2 minutes.

   + If no message or files or directories found, the parsing will fail.

   + If parsing fails, exit.

2. Sender binds on a UDP server to listen to it.

    + the port number is assigned by the OS;

    + if error happens, exit.

3. Sender generates a code which is actually the UDP port number.

4. Sender starts `timer` based on the expire time. 

    + If time runs out, exit.

    + Timer can be terminated if a connection is established.

5. Receiver parses command arguments as below:

`isend -r -p --overwrite r your_recv_code`

   + If `-p` is specified, a new line will prompt to ask for the password.

   + Overwrite strategy could be "o"(overwrite), "r"(rename) or "s"(skip)

   + If parsing fails, exit.

6. Receiver starts listening on a TCP port:

    + the TCP port number is assigned by the OS;

    + if error happens, exit.

7. Receiver sends UDP broadcast based on the receiving code.

    + The broadcast will repeat every 5 seconds;

    + Exit when reach the `retry` limit, curretly set as 10.

8. Sender receives the broadcast and try to connect to receiver's TCP socket with password if specified.

    + Valid password => sender stops timer and continue to step 9;

    + Invalid => ignore it.

9. Sender starts sending contents after the connection being established:

    + Send files/directories if exists;

    + Send message if exists;

    + Send Disconnect request.

10. Receiver starts receiving request and contents in a loop:

    + Sending dir request => Checking dir name, overwrite strategy, and create dir if necessary;

    + Sending file request => Checking file name, overwrite strategy, and receive file contents;

    + Sending message request => Receive message string and display it;

    + Disconnect request => break the loop and exit.