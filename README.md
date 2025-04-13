# TermEdit--Bad-Terminal-Editor
A bad terminal based code editor created in rust for rust

## Custom Escape Codes (using iTerm2 for custom key-bindings):
 - ^[[3;22~  (⌥ Tab)
 - ^[[3;21~  (⌘ ⇧ 'z')
 - ^[[3;16~  (⌘ 'c')
 - ^[[3;19~  (⌘ 'f')
 - ^[[3;11~  (⌘ 's')
 - ^[[3;17~  (⌘ 'v')
 - ^[[3;18~  (⌘ 'x')
 - ^[[3;20~  (⌘ 'z')
 - ^[[3;2~   (⇧ Delete)
 - ^[[3;3~   (⌥ Delete)
 - ^[[3;8~   (⌥ ⇧ Delete)
 - ^[[3;9~   (⌘ Delete)
 - ^[[3;10~  (⌘ ⇧ Delete)
 - ^[[3;6~   (⌘ ^)
 - ^[[3;14~  (⌘ ⇧ ^)
 - ^[[3;7~   (⌘ v)
 - ^[[3;15~  (⌘ ⇧ v)
 - ^[[3;4~   (⌘ <-)
 - ^[[3;12~  (⌘ ⇧ <-)
 - ^[[3;5~   (⌘ ->)
 - ^[[3;12~  (⌘ ⇧ ->)

Without these codes/bindings, the program will still work, although special actions (option/command/shift + certain keys) will be limited due to the terminal not sending anything for the events/actions.

The terminal may also need to be in xterm/xterm-256 color or some mode like that.

Depending on the terminal, some other settings or configurations may be necessary to get full support for these additional bindings.

All mouse events should be standardized so as long as the terminal is sending them in the correct format, it should work fine.
