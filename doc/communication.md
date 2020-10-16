Communication between UI and Model
---

### UI -> Model

Arguments: SendArg or RecvArg;

Choice: a string

### Model -> UI

+ Notification: 

    + Success connection

    + Transmitting file

+ Progress:

    + File transfer progress <with a number, update dynamic>

+ Confirmation:

    + Overwrite? <Needs a response>

+ Error: <leads to the process exits>

    + Connection refused

    + Other errors