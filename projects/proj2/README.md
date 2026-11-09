# Jak uruchomić

Aby zbudować odpal

```
make
```

Wygeneruje on pliki binarne w `target/release`.

Uruchom główny program na załączonej przykładowej bazie:

```
./target/release/proj2 sample.db
```

Plik bazy został wygenerowany przez:

```
./target/release/gen NAZWA.db
```

Kilka testów end-to-end można uruchomić przez:

```
make test
```
