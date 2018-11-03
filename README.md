# nep

the editor of choice for late teen losers who are too edgy for vim and too lazy for emacs

## running

make sure you have `cargo` installed 

```
git clone https://github.com/nep-editor/nep.git
cd nep
cargo install
nep
```
(note: current build does not allow you to save if you open without a file name)

## development plan (ish)

- [x] make self hosting (03/11/18)
- [ ] make modal (05/11/18)
- [ ] implement buffer system (09/11/2018)
  - [ ] buffer transactions
  - [ ] asynchronous write-back
  - [ ] undo for christs sake
- [ ] implement multiple cursors over buffers (11/11/18)
- [ ] build multiple buffer rendering system (19/11/18)
- [ ] design module system (22/11/18)
- [ ] implement module system (29/11/18)
- [ ] plan further development
