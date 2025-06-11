# TermEdit
TermEdit is a lightweight code editor that runs entirely within the terminal environment. The terminal-based environment allows improved portability, reduced CPU and RAM usage, improved responsiveness, and wide-ranging support. Utilizing a custom parallel and async rendering framework and event manager, the application idles and runs at around 0.1% to 8.5% CPU usage on a Mac M1, using around 5mb to 25mb of RAM. The editor also supports syntax highlighting, which is entirely customizable through dynamically executed Lua scripts. The overarching goal of this editor was to offer some more modern IDE-like features while remaining lightweight, utilizing as little RAM as possible, and having immediate runtime response to inputs and code changes. This goal was derived from a frustration with modern IDEs using vast amounts of RAM, containing countless memory leaks, and taking ages to update changes or offer suggestions (to the point where the line would be completed before it'd finish generating the suggestion).

> ⚠ The editor is still under development and may not be stable. Bug fixes and polishes are still in the works, along with greater feature support. This project is entirely solo right now, so development will only progress so quickly.

## Features
The editor has a limited settings selection at this time (plans to expand that selection and offer more customization are in the works). Currently, there's a setting for preferred keybindings (command key or control key) and the color type (some terminals don't support 8-bit colors, so there are multiple options to ensure support). The editor also offers a clean work environment with undisturbed code panes (the editor supports multiple concurrent panes/split screen). Suggestions and errors in the future are presented in a box at the bottom, which is visible but out of the way so as not to interrupt the programming environment. Additionally, certain keybindings, like tab for auto complete (now option + tab), are different to allow a more seamless programming experience without accidentally pressing an undesired key combination. The editor also supports having multiple tabs opened at once that can be quickly switched between while still meeting the strict RAM and CPU usage goals. That multi-tab system is also compounded by a quick load-time on large files (including parsing and lexing the file/files).
 
The editor's architecture utalizes two different lexers to enable instant responces to code changes and general responsiveness without any studders on large files. The first lexer is a partial lexer which handles syntax highlighting on a per-line basis allowing limited recalculation when rapidly editing files. The second is a full lexer which anlyzes the structure and components of the program allowing for basic auto-compelte suggestions based on variable names, enum variants, etc... (this lexer is still under development and not fully stable). The full lexer runs on background threads, sent out sparingly while still ensuring quick but low-cost recalculations. The full lexer is also statically linked as a package through compiletime procedural macros. An interface is also defined through a trait allowing for an easy drag and drop system for adding new language support. This builds upon the already implimented system for syntax highlighting; all of these use the very simply formated syntax highlighting json file to configure the linking.
 
The rendering framework uses a caching system that handles windows as unique structs, which are stored for a duration of time rather than recreated each frame, unlike in Ratattui (a great Rust terminal UI library, although heavier weight, and doesn't easily permit caching). Those windows are set only to update each line when necessary, allowing lazy and deferred rendering, ensuring low idle times. Escape codes are also combined when possible, and other escape-code-oriented optimizations are in place. The windows (widgets) send out ECS-style closures for any segments needing rerendering, allowing for the rendering and stylization calculation to be calculated entirely on a background thread while avoiding the overhead of atomic containers like Arc and read-write blocks like parking_lot::RwLock (there's also a std::sync::RwLock).
> * go to https://rust-analyzer.github.io/book/rust_analyzer_binary.html for information on installing the rust-analyzer LSP binary (the editor will soon support it).

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
 - (->, <-, v, ^ represent directional arrow keys)
 - probably others that I forgot...

> Without these codes/bindings, the program will still work, although special actions (option/command/shift + certain keys) will be limited due to the terminal not sending anything for the events/actions.
> * Switch to Control as the default modifier key to get a wider support for default escape codes (control + key is much more widely supported in terminals).

The terminal may also need to be in xterm/xterm-256 color or some mode like that.

Depending on the terminal, some other settings or configurations may be necessary to get full support for these additional bindings.

All mouse events should be standardized so as long as the terminal is sending them in the correct format, it should work fine.


If the text or color rendering is messed up, make sure to go into settings on the main menu and change the color type setting. Pretty much all terminals support the base 7 ASCII colors. Most support the ANSI colors. Only a few support 8-bit color.

 - Type q to quit
 - Type -light or -dark to change the color theme
