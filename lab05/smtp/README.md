# Как запустить?

```shell
./gradlew --console=plain -q run --args="..."
```

Аргументы:
```
Options: 
    --client [simple] -> type of smtp client. Either "simple" or "socket" value { String }
    --from -> sender email address (always required) { String }
    --username -> sender username (always required) { String }
    --password -> sender email password (always required) { String }
    --to -> recipient email address (always required) { String }
    --message-filepath -> path file with message (always required) { String }
    --message-subject [Message subject] -> message subject { String }
    --server-addr [smtp.yandex.ru] -> smtp server address { String }
    --server-port [587] -> smtp server port { Int }
    --help, -h -> Usage info 
```