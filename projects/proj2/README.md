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

Testy można uruchomić przez:

```
make test
```

# Format pliku

Format binarny rozpoczynamy następującymi metadanymi:

- liczba wierszy : u64
- liczba kolumn : u64
- rodzaj kolumny: `[u8; liczba_kolumn]`

Następnie mamy serializację kolejnych batchy (każdy maks. 1024 wiersze):

- batch 1
- batch 2
- ...
- batch m

Każdy batch składa się z serializacji kawałków kolejnych kolumn:

- kolumna 1
- kolumna 2
- ...
- kolumna k

Gdzie każda kolumna rozpoczyna się u64 przechowującym jej długość w bajtach, dzięki czemu można szybko
przeskoczyć do n-tej kolumny bez parsowania poprzednich. Następnie:

- dla booli mamy dump bitów
- dla intów jest VLE delt, gdzie liczby ze znakiem są kodowane za pomocą techniki `zigzag`
- napisy najpierw mają VLE delt długości (zakładam, że napisy w danej kolumnie są podobnej długości),
  a następnie kompresję konkatenacji wszystkich napisów za pomocą zstandard
