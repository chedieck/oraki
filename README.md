# Oraki

Search queries on [OpenRussian](en.openrussian.org) and create an Anki deck with the results.


Usage
---

After installing the program, running for example:
```
$ oraki нежны
```
will output:

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

Anki
---
Every search you make is saved on `$HOME/.local/share/oraki/main.csv`. This file can then be parsed by `oraki` to create an anki deck by running `oraki --compile`, or simply `oraki -c`. Here is the example of a card:

<p align="center">Front:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/225175364-a25f318a-4ee7-4068-b240-d68d0eec9f65.png">
</p>
<p align="center">Back:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/225175388-98f0cb8e-4c26-4b1e-b6d7-4cd81deebeea.png">
</p>

Things to notice:
- The card "question" is the result of the search;
- The answer in the back includes the stressed syllable;
- In parenthesis is the search query.


Context phrase
---

Maybe you want to have a phrase to give context to the searched word. All other arguments after the search query are parsed as the context phrase, so you can run something like: `oraki красивая Она очень красивая девушка.`. Here is the result

<p align="center">Front:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/225175233-95878e2a-54e5-4341-8470-d055c9b3f29b.png">
</p>
<p align="center">Back:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/225175200-2f41411b-6495-470a-b3b8-8e2161a74f58.png">
</p>




The deck is saved in `$HOME/.local/share/oraki/output.apkg`. You can then import the file with anki, study it, make new search queries with `oraki <query>`, export it again with `oraki -c`, import it again on Anki and it will update the deck with the new cards.

For a more simple approach into creating Anki cards from sentences, you can run `oraki -f path/to/file`. An example of such a file is available at `extra/example.list`.

Configuration
---
There is a CSS file at `extra/style.css`, you can customize it together with the HTML constants `Q_FORMAT` and `A_FORMAT` at `src/anki.rs`, to change the cards style, and update it with `make install`.


Installing:
---
Clone the repo, go into the directory and run `make install`.


TODO
---
- [ ] Save all main translations, not the first one
- [x] Create function to do many queries from a file
- [ ] Get context phrase when not provided and it exists
