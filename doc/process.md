Isend workflow
---

1. Sender parses command arguments as below:

`isend -s -p -e 10 -m Hello --port 30000 --shutdown ~/hello.txt`

   + If `-p` is specified, a new line will prompt to ask for the password.

   + If parsing fails, exit.

2. Send starts a UDP server and listening on the port

    + --port = 0 =>  assigned by system;

    + --port > 0 => specified by user;

    + if error happens, exit.

3. Sender generates a 6 character code, based on its open UDP port and a random password, then displays the code on the screen.

4. Sender starts `timer` speicifed by command `-e`. 

    + If time runs out, exit.

    + Timer can be terminated by instruction from other funcion if a connection is established.

5. Receiver parses command arguments as below:

`isend -r -p -d ~/Downloads --port 40000 --shutdown 092c3f -o a`

   + If `-p` is specified, a new line will prompt to ask for the password.

   + If parsing fails, exit.

6. Receiver starts listening on a TCP port:

    + --port = 0 =>  assigned by system;

    + --port > 0 => specified by user;

    + if error happens, exit.

7. Receiver modifies the first 4 characters of the code by chaning it to its own TCP port, and send UDP broadcast with the new code.

    + The broadcast will repeat every 5 seconds;

    + Exit when reach the `retry` limit.

8. Receiver starts counting time:

    + If time runs out, exit

    + Timer can be terminated by instruction from other function if a connection is established.

9. Sender receives the broadcast and check against the password from the last two characters of the code.

    + Valid password => start step 10;

    + Invalid => ignore it.

10. Sender resets the `timer` and sends connection request to the receiver.

    + If a password is specified, it should be sent after the request.

    + If time runs out, exit.

11. Receiver receive the connection request:

    + If password required => Read password from next buffer;

        + Password matches => Go to next step;

        + Password unmatched => replies `connection refused` and continue listening.
    
    + No password required => Go to next step.

12. Receiver resets the `timer` and replies `connection success`.

13. Sender receives replies:

    + Connection success => go to next step;

    + Connection refused => exit.

14. Sender sends message if any:

    + one packet for operation request;

    + one packet for message content.

    + Every packet id needs to be stored into a collection.