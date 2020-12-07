Current usage
---

Sender side:

```
isend -s -e 10 -p a.txt b.png ~/Documents/ -m "Hello from isend"
```

Receiver side:

```
isend -r -p --overwrite o your_recv_code
```

Help (or check `src/cli/cli.yaml` for details):

```
isend --help
```