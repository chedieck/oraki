# Oraki

Search terms on OpenRussian.org and create an Anki deck with the results.


# Summary

After installing the program, running for example:

```
$ oraki нежны
```
will output something like:

```
не́жный
tender
(delicate, loving, affectionate, fond)
--------------------------------------
adjective
adverb не́жно
somewhat often used word (top 3,000)
```

You can search in english too:
```
$ oraki knife
```
will output:
```
нож
knife
(table-knife, penknife, clasp-knife)
------------------------------------
noun, masculine, inanimate
somewhat often used word (top 2,000)
```

# Anki
Every search you make is saved on `$HOME/.local/share/oraki/main.csv`. This file can then be parsed by `oraki` to create an anki deck by running `oraki --compile`, or simply `oraki -c`.

The deck is saved in `$HOME/.local/share/oraki/output.apkg`. You can then import the file with anki, study it, make new search queries with `oraki <query>`, export it again with `oraki -c`, import it again on Anki and it will update the deck with the new cards.

# Configuration
There is a CSS file at `$HOME/.local/share/oraki/style.css`, you can customize it together with the constants `Q_FORMAT` and `A_FORMAT`, to change the question and answer style, respectively.


Installing:
---
Clone the repo, go into the directory and run `make install`.


TODO
---
- [ ] Save all main translations, not the first one
