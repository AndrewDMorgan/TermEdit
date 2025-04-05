use tokio::io::AsyncWriteExt;
use vte::Perform;

#[derive(PartialEq)]
pub enum KeyModifiers {
    Shift,
    Command,
    Option,
    Control,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeyCode {
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Return,
    Escape,
}

pub enum MouseEventType {
    Null,
    Left,
    Right,
    Middle,
    Down,
    Up,
}

pub enum MouseState {
    Release,
    Press,
    Hold,
    Null,
}

pub struct MouseEvent {
    pub eventType: MouseEventType,
    pub position: (u16, u16),
    pub state: MouseState,
}

#[derive(Default)]
pub struct KeyParser {
    pub keyModifiers: Vec <KeyModifiers>,
    pub keyEvents: std::collections::HashMap <KeyCode, bool>,
    pub charEvents: Vec <char>,
    pub inEscapeSeq: bool,
    pub bytes: usize,
    pub mouseEvent: Option <MouseEvent>,
    pub mouseModifiers: Vec <KeyModifiers>,
    pub lastPress: u128,
}

impl KeyParser {
    pub fn new () -> Self {
        KeyParser {
            keyEvents: std::collections::HashMap::from([
                (KeyCode::Delete, false),
                (KeyCode::Tab, false),
                (KeyCode::Left, false),
                (KeyCode::Right, false),
                (KeyCode::Up, false),
                (KeyCode::Down, false),
                (KeyCode::Return, false),
                (KeyCode::Escape, false),
            ]),
            keyModifiers: vec!(),
            charEvents: vec!(),
            inEscapeSeq: false,
            bytes: 0,
            mouseEvent: None,
            mouseModifiers: vec!(),
            lastPress: 0,
        }
    }

    pub fn ClearEvents (&mut self) {
        self.charEvents.clear();
        self.keyModifiers.clear();
        self.mouseModifiers.clear();
        self.keyEvents.clear();
        self.inEscapeSeq = false;

        if let Some(event) = &mut self.mouseEvent {
            match event.state {
                MouseState::Press => {
                    event.state = MouseState::Hold;
                },
                MouseState::Hold if matches!(event.eventType, MouseEventType::Down | MouseEventType::Up) => {
                    event.state = MouseState::Release;
                },
                MouseState::Release => {
                    event.state = MouseState::Null;
                    event.eventType = MouseEventType::Null;
                },
                MouseState::Hold => {
                },
                _ => {},
            }
        }
    }

    pub fn ContainsChar (&self, chr: char) -> bool {
        self.charEvents.contains(&chr)
    }

    pub fn ContainsModifier (&self, modifier: KeyModifiers) -> bool {
        self.keyModifiers.contains(&modifier)
    }

    pub fn ContainsMouseModifier (&self, modifier: KeyModifiers) -> bool {
        self.mouseModifiers.contains(&modifier)
    }

    pub fn ContainsKeyCode (&self, key: KeyCode) -> bool {
        *self.keyEvents.get(&key).unwrap_or(&false)
    }

}

pub async fn enableMouseCapture() {
    let mut stdout = tokio::io::stdout();
    let _ = stdout.write_all(b"echo -e \"\x1B[?1006h").await;
    let _ = stdout.write_all(b"\x1B[?1000h").await; // Enable basic mouse mode
    let _ = stdout.write_all(b"\x1B[?1003h").await; // Enable all motion events
    std::mem::drop(stdout);
}

pub async fn disableMouseCapture() {
    let mut stdout = tokio::io::stdout();
    let _ = stdout.write_all(b"\x1B[?1000l").await; // Disable mouse mode
    let _ = stdout.write_all(b"\x1B[?1003l").await; // Disable motion events
    std::mem::drop(stdout);
}

impl KeyParser {
    pub fn SetPressTime (&mut self) {
        self.lastPress = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_millis();
    }
}

impl Perform for KeyParser {
    fn execute(&mut self, byte: u8) {
        self.SetPressTime();
        match byte {
            0x1B => {
                self.inEscapeSeq = true;
            },
            0x0D => {  // return aka \n
                self.keyEvents.insert(KeyCode::Return, true);
            },
            0x09 => {
                self.keyEvents.insert(KeyCode::Tab, true);
            },
            _ => {},
        }
        //println!("byte {}: '{}'", byte, byte as char);
    }

    fn print(&mut self, chr: char) {
        if self.inEscapeSeq || self.bytes > 1 {  return;  }
        self.SetPressTime();

        if chr as u8 == 0x7F {
            self.keyEvents.insert(KeyCode::Delete, true);
            return;
        }
        if !(chr.is_ascii_graphic() || chr.is_whitespace()) {  return;  }
        //println!("char {}: '{}'", chr as u8, chr);
        self.charEvents.push(chr);
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _: &[u8], _: bool, c: char) {
        self.inEscapeSeq = false;  // resetting the escape sequence
        self.SetPressTime();

        let numbers: Vec<u16> = params.iter().map(|p| p[0]).collect();

        // mouse handling
        if c == 'M' || c == 'm' {
            if let Some([byte, x, y]) = numbers.get(0..3) {
                let button = byte & 0b11; // Mask lowest 2 bits (button type)
                //println!("button: {}, numbers: {:?}", button, numbers);

                // adding key press modifiers
                if (byte & 32) != 0 {
                    self.keyModifiers.push(KeyModifiers::Shift);
                }
                if (byte & 64) != 0 {
                    self.keyModifiers.push(KeyModifiers::Option);
                }
                if (byte & 128) != 0 {
                    self.keyModifiers.push(KeyModifiers::Control);
                }

                //println!("Code: {:?} / {}", numbers, c);

                let isScroll = (byte & 64) != 0;
                let eventType = match (isScroll, button) {
                    (true, 0) => MouseEventType::Up,   // 1???? ig so
                    (true, 1) => MouseEventType::Down, // 2???? ig so
                    (false, 0) => MouseEventType::Left,
                    (false, 1) => MouseEventType::Middle,
                    (false, 2) => MouseEventType::Right,
                    _ => MouseEventType::Null
                };

                if matches!(eventType, MouseEventType::Left) && numbers[0] == 4 {
                    self.mouseModifiers.push(KeyModifiers::Shift);
                }

                if let Some(event) = &mut self.mouseEvent {
                    if matches!(eventType, MouseEventType::Left) &&
                        event.position != (*x, *y) &&
                        matches!(event.state, MouseState::Hold) &&
                        c == 'M'
                    {
                        event.position = (*x, *y);
                        return;
                    }
                }

                self.mouseEvent = Some(MouseEvent {
                    eventType,
                    position: (*x, *y),
                    state: {
                        match c {
                            'M' => MouseState::Press,
                            'm' => MouseState::Release,
                            _ => MouseState::Null,
                        }
                    },
                });

            }

            return;
        }

        //for number in &numbers {println!("{}", number);}
        if c == '~' {  // this section is for custom escape codes
            if numbers == [3, 2] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 3] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
            } else if numbers == [3, 4] {
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 5] {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 6] {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 7] {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 8] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 9] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 10] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 11] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('s');  // command + s
            } else if numbers == [3, 12] {  // lrud
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 13] {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 14] {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 15] {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 16] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('c');
            } else if numbers == [3, 17] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('v');
            } else if numbers == [3, 18] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('x');
            } else if numbers == [3, 19] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('f');
            } else if numbers == [3, 20] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('z');
            } else if numbers == [3, 21] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
                self.charEvents.push('z');
            } else if numbers == [3, 22] {
                self.keyEvents.insert(KeyCode::Tab, true);
                self.keyModifiers.push(KeyModifiers::Option);
            }
        } else {  // this checks existing escape codes of 1 parameter/ending code (they don't end with ~)
            match c as u8 {
                0x5A => {
                    self.keyEvents.insert(KeyCode::Tab, true);
                    self.keyModifiers.push(KeyModifiers::Shift);
                },
                0x44 => {
                    self.keyEvents.insert(KeyCode::Left, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x43 => {
                    self.keyEvents.insert(KeyCode::Right, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x41 => {
                    self.keyEvents.insert(KeyCode::Up, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x42 => {
                    self.keyEvents.insert(KeyCode::Down, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                _ => {},
            }
        }
    }
}

