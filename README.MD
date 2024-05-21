# Goal
A replacament for: `cd`, `ls`, `cp`. `mv`, `rm` with an asynchronous preview window

# Features

## Preview Window
A vertical split that previews the cursor entry

The preview pane should be *asynchronous* with the navigation pane so it won't slow down the navigation on rendering big directories/files

### Directory
Show entries inside the directory in the right split
The user should be able to navigate to the preview directory
and make it the current one or directly open a file inside it.
```
 +--------------------|--------------------+
 |~/example           |~/example/folder    |
 |➜ 📁 folder         |- 📁 anotherfolder  |
 |- 📁 folder_empty   |- 📄 anotherfile    |
 |- 🖼️ image.png      |                    |
 |- 📄 text.txt       |                    |
 |                    |                    |
 |                    |                    |
 |                    |                    |
 +--------------------|--------------------+
```

### Text
[Text/Code Preview](https://github.com/sharkdp/bat)

```
 +--------------------|--------------------+
 |~/example           |File: text.txt      |
 |- 📁 folder         |                    |
 |- 📁 folder_empty   |the quick brown fox |
 |- 🖼️ image.png      |jumps over the lazy |
 |➜ 📄 text.txt       |dog                 |
 |                    |                    |
 |                    |                    |
 |                    |                    |
 |                    |                    |
 +--------------------|--------------------+
```
## Image
[ASCII Preview](https://github.com/lnenad/image-to-ascii)

```
 +--------------------|--------------------+
 |~/example           |      @@@@@@@       |
 |- 📁 folder         |    @@@@@@@@@@@     |
 |- 📁 folder_empty   |   @@@@@@@@@@@@@    |
 |➜ 🖼️ image.png      |  @@@@@@@@@@@@@@@   |
 |- 📄 text.txt       |  @@@@@@@@@@@@@@@   |
 |                    |  @@@@@@@@@@@@@@@   |
 |                    |   @@@@@@@@@@@@@    |
 |                    |    @@@@@@@@@@@     |
 |                    |      @@@@@@@       |
 +--------------------|--------------------+
```
