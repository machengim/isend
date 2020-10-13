ISend Protocol
---

### Send file process:

+ START
+ Sender requests to send file with filename length <request code 10><followed by 1 byte for length>
+ Receiver prepares buffer and confirms <reply needed?>
+ Sender sends filename	<content max length 256>
+ Receiver creates file with the filename and confirms <reply code>
	Options may appear here (overwrite/skip/rename);
	In skip mode, an instruction should be replied to abort this operation.
Repeat:
	+ Sender requests to send file content with content size <request code 11><followed by 2 byte length: max 64k>
	+ Receiver prepares buffer and confirms
	+ Sender sends content <content max size 64k? or 32k? considering max TCP window size is 64k>
	+ Receiver receives and confirms
+ Sender sends notification that file content end <request code 12>
+ Receiver confirms. <reply code needed>
+ END.

### Send directory process:

+ START
+ Sender requests to send directory with dir name length <request code 20><followed by 1 byte length>
+ Receiver prepares buffer and confirms
+ Sender send dir name <content max length 256>
+ Receiver create dir and confirms <reply code>
	Options may appear here (merge/skip/rename policy);
	In skip mode, an instruction should be replied to abort this operation.
	In other mode, change working dir to this new dir
Repeat:
	Send file process
+ Sender sends notification that directory content end <request code 21>
+ Receiver changes working dir to original one and confirms. <reply code needed>
+ END

### Send message process:

+ START
+ Sender requests to send message with a length <request code 30><followed by 1 byte length>
+ Receiver prepares buffer and confirms
+ Sender sends content <content max length 256>
+ Receiver receives and confirms <reply code needed>
+ END

### Connect without password process: (this happens before the connection starts)
+ START
+ Sender requests to send password with a length <request code 0>
+ Receiver confirms<reply code needed>
	Request may be refused if the receiver has set up a password. <reply code needed>
	In this case, the receiver will shutdown
+ END

### Connect without password process: (this happens before the connection starts)
+ START
+ Sender requests to send password with a length <request code 1><followed by 1 byte length>
+ Receiver prepares buffer and confirms <reply code needed>
+ Sender sends password <content max length 256>
+ Receiver receives and confirms <reply code needed>
	Request may be refused if the receiver has set up a password. <reply code needed>
+ END

---

### Request instruction structure: <6 bytes in total>
+ Request id (2 byte)
+ operation code (1 byte)
+ has content (1 byte)
+ buffer length (2 bytes)

### Reply instruction structure <Same>: 
+ Request id (2 byte)
+ operation code (1 byte)
+ has content (1 byte)
+ buffer length (2 bytes)

### Request operation code:
+ Prepare connection without password: 0
+ Prepare connection with password: 1

+ Prepare sending file: 10
+ Prepare seding file content: 11
+ End sending file: 12

+ Prepare sending dir: 20
+ End sending dir: 21

+ Prepare sending message: 30

### Reply operation code: (Is it necessary to make these code more specific?)
+ Connection established: 101
+ Connection refused: 102
+ Connection failed: 103

+ Request confirmed: 110
+ Request refused: 111
+ Request failed: 112

+ Abort current file transmission: 120
+ Abort current dir transmission: 121

### Common code:
+ End connection: 255

Reply may have a message along with it, and that's the reason for requesting a buffer on sender's side.
