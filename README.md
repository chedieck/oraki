# Oraki

Search queries on [OpenRussian](en.openrussian.org) and create an Anki deck with the results.


Usage
---

After installing the program, running for example:
```
$ oraki нежны
```
will output something like:

```
┌────────────────────────────────────────┐
│                 не́жный                 │
│                 tender                 │
│ (delicate, loving, affectionate, fond) │
│────────────────────────────────────────│
│               adjective                │
│              adverb не́жно              │
│  somewhat often used word (top 3,000)  │
└────────────────────────────────────────┘

Сча́стье - не́жный цвето́к.
Happiness is a delicate flower.
```
**You can search in english too:**
```
$ oraki knife
```
will output:
```
┌──────────────────────────────────────┐
│                 нож                  │
│                knife                 │
│ (table-knife, penknife, clasp-knife) │
│──────────────────────────────────────│
│      noun, masculine, inanimate      │
│ somewhat often used word (top 2,000) │
└──────────────────────────────────────┘

Ле́звие э́того ножа́ очень о́строе.
The knife has a keen blade.
```

Every search is saved so that later you can create a Anki deck with them. For that reason, you can also run oraki on a list of words with  `oraki -f path/to/file`. An example of such a file is available at `extra/example.list`.

Anki
---
Every search you make is saved on `$HOME/.local/share/oraki/main.csv`. This file can then be parsed by `oraki` to create an anki deck by running `oraki --compile`, or simply `oraki -c`. Here is the example of a card:

<p align="center">Front:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/226424934-e5a8e555-f893-453b-b0ac-a26539d80d23.png">
</p>
<p align="center">Back:</p>
<p align="center">
<img width="600" src="https://user-images.githubusercontent.com/21281174/226424381-160d7719-f9b1-48b6-80f4-d3d223f8b10e.png">
</p>




Things to notice:
- The card "question" is the result of the search + the russian phrase if it exists;
- The answer in the back has the word with the stressed syllable marker right before the search query (in parenthesis)

The deck then is saved on `~/.local/share/oraki/output.apkg` and can simply be imported to anki. Every time you do that, old cards will mantain their data, new ones will be added.


Configuration
---
There is a CSS file at `extra/style.css`, you can customize it together with the HTML constants `Q_FORMAT` and `A_FORMAT` at `src/anki.rs`, to change the cards style, and update it with `make install`.


Installing:
---
Clone the repo, go into the directory and run `make install`.


TODO
---
- [x] Create function to do many queries from a file
- [x] Get context phrase when not provided and it exists
- [ ] Remove support for custom phrase
