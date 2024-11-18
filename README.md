# Revo

![screenshot dark](./assets/dark_ss.png)
![screenshot light](./assets/light_ss.png)

## Milestones

- [x] Double Buffering in Terminal
- [x] Save & Save As
- [x] Basic Settings and Theme support
- [ ] Buffer Decorations
- [ ] Split Support
- [ ] File Explorer
- [ ] General UX
- [ ] Code clean up + Tests

## Bugs


## Improvements

- Can reduce string allocation for line numbers in `draw_buffer` by allocating one string 
  before the loop, and reusing it inside.

- Update `BufferData` API to be self contained. So that in the future if we decide to
  change the implementation of how we represent the data, nothing else needs to be updated.

- Update active buffer width and height on resize.