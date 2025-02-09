use crate::support::CapBuilder;
use std::io::prelude::*;
use std::sync::mpsc::Receiver;
use term::terminfo::TermInfo;
use vterm_sys::*;

#[test]
fn state_can_generate_put_glyph_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());
    vterm.write(b"a").unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_put_glyph_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.glyph_info.chars[0], b'a');
    assert_eq!(event.pos.x, 0);
    assert_eq!(event.pos.y, 0);
}

#[test]
fn state_can_generate_move_cursor_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("cup")
                .number_param(0)
                .number_param(1)
                .build()
                .unwrap(),
        )
        .unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_move_cursor_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.new, Pos { x: 1, y: 0 });
    assert_eq!(event.old, Pos { x: 0, y: 0 });
    assert_eq!(event.is_visible, true);
}

#[test]
fn state_can_generate_scroll_rect_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());
    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("ri").build().unwrap())
        .unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_scroll_rect_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.rect, Rect::new(Pos::new(0, 0), Size::new(2, 2)));
    assert_eq!(event.downward, -1);
    assert_eq!(event.rightward, 0);
}

#[test]
fn state_can_generate_scroll_rect_events_for_part_of_screen() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 5,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());
    let terminfo = TermInfo::from_name("xterm").unwrap();

    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("csr")
                .number_param(3)
                .number_param(5)
                .build()
                .unwrap(),
        )
        .unwrap();

    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("cup")
                .number_param(3)
                .number_param(0)
                .build()
                .unwrap(),
        )
        .unwrap();

    vterm
        .write(&CapBuilder::new(&terminfo).cap("ri").build().unwrap())
        .unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_scroll_rect_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.rect, Rect::new(Pos::new(0, 3), Size::new(2, 2)));
    assert_eq!(event.downward, -1);
    assert_eq!(event.rightward, 0);
}

#[test]
fn state_can_generate_move_rect_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();

    let mut config = StateCallbacksConfig::all();
    config.scroll_rect = false;
    vterm.state_receive_events(&config);

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("ri").build().unwrap())
        .unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_move_rect_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.src, Rect::new(Pos::new(0, 0), Size::new(2, 1)));
    assert_eq!(event.dest, Rect::new(Pos::new(0, 1), Size::new(2, 1)));
}

#[test]
fn state_can_generate_move_rect_events_for_part_of_screen() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 5,
        width: 2,
    })
    .unwrap();

    let mut config = StateCallbacksConfig::all();
    config.scroll_rect = false;
    vterm.state_receive_events(&config);

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("csr")
                .number_param(2)
                .number_param(5)
                .build()
                .unwrap(),
        )
        .unwrap();

    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("cup")
                .number_param(2)
                .number_param(0)
                .build()
                .unwrap(),
        )
        .unwrap();

    vterm
        .write(&CapBuilder::new(&terminfo).cap("ri").build().unwrap())
        .unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_move_rect_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.src, Rect::new(Pos::new(0, 2), Size::new(2, 2)));
    assert_eq!(event.dest, Rect::new(Pos::new(0, 3), Size::new(2, 2)));
}

#[test]
fn state_can_generate_erase_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();

    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSED
    vterm.write(b"\x1b[?2J").unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_erase_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.rect, Rect::new(Pos::new(0, 0), Size::new(2, 2)));
    assert_eq!(event.is_selective, true);
}

#[test]
fn state_can_generate_init_pen_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_init_pen_event(&rx);

    assert!(event.is_some());
}

#[test]
fn state_can_generate_pen_background_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("setb")
                .number_param(1)
                .build()
                .unwrap(),
        )
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_background_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    // Why wouldn't this be 1?
    assert_eq!(event.palette, 4);
}

#[test]
fn state_can_generate_pen_blink_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("blink").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_blink_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_blink_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_pen_bold_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("bold").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_bold_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_bold_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_pen_font_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    vterm.write(b"\x1b[10m").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_font_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.font, 0);

    vterm.write(b"\x1b[12m").unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_font_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.font, 2);
}

#[test]
fn state_can_generate_pen_foreground_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("setf")
                .number_param(3)
                .build()
                .unwrap(),
        )
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_foreground_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    // Why isn't this 3? Seems like my xterm terminfo is putting my values out of sequence.
    assert_eq!(event.palette, 6);
}

#[test]
fn state_can_generate_pen_italic_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    // SITM and SGR isn't supported. Just do it manually
    vterm.write(b"\x1b[3m").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_italic_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_italic_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_pen_reverse_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("sgr")
                .number_param(0)
                .number_param(0)
                .number_param(1)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .build()
                .unwrap(),
        )
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    // Need to flush out the first one.
    // TODO: make the try_recv_* fns return a vec then assert on the Vec?
    try_recv_pen_reverse_event(&rx);
    let event = try_recv_pen_reverse_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_reverse_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_pen_strike_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm.write(b"\x1b[9m").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_pen_strike_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_strike_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_pen_underline_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(
            &CapBuilder::new(&terminfo)
                .cap("sgr")
                .number_param(0)
                .number_param(1)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .number_param(0)
                .build()
                .unwrap(),
        )
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    try_recv_pen_underline_event(&rx);
    let event = try_recv_pen_underline_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.mode, Underline::Single);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("sgr0").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event = try_recv_pen_underline_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.mode, Underline::None);
}

#[test]
fn state_can_generate_cursor_visible_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("civis").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<CursorVisibleEvent> = try_recv_cursor_visible_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("cnorm").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event: Option<CursorVisibleEvent> = try_recv_cursor_visible_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);
}

#[test]
fn state_can_generate_cursor_blink_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSCUSR
    vterm.write(b"\x1b[1 q").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<CursorBlinkEvent> = try_recv_cursor_blink_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm.write(b"\x1b[2 q").unwrap();
    vterm.flush().unwrap();

    let event: Option<CursorBlinkEvent> = try_recv_cursor_blink_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_cursor_shape_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSCUSR sequence
    vterm.write(b"\x1b[4 q").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<CursorShapeEvent> = try_recv_cursor_shape_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.shape, CursorShape::Underline);

    vterm.write(b"\x1b[0 q").unwrap();
    vterm.flush().unwrap();

    let event: Option<CursorShapeEvent> = try_recv_cursor_shape_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.shape, CursorShape::Block);
}

#[test]
fn state_can_generate_title_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSWT
    vterm.write(b"\x1b]2;foo\x1b\\").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<TitleEvent> = try_recv_title_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.title, "foo");

    vterm.write(b"\x1b]2;bar\x1b\\").unwrap();
    vterm.flush().unwrap();

    let event: Option<TitleEvent> = try_recv_title_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.title, "bar");
}

#[test]
fn state_can_generate_iconname_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSWT
    vterm.write(b"\x1b]1;foo\x1b\\").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<IconNameEvent> = try_recv_icon_name_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.name, "foo");

    vterm.write(b"\x1b]1;bar\x1b\\").unwrap();
    vterm.flush().unwrap();

    let event: Option<IconNameEvent> = try_recv_icon_name_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.name, "bar");
}

#[test]
fn state_can_generate_reverse_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSCNM
    vterm.write(b"\x1b[?5h").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<ReverseEvent> = try_recv_reverse_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm.write(b"\x1b[?5l").unwrap();
    vterm.flush().unwrap();

    let event: Option<ReverseEvent> = try_recv_reverse_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_mouse_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // DECSET for mouse support
    vterm.write(b"\x1b[?1003h").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<MouseEvent> = try_recv_mouse_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();

    assert_eq!(event.mode, MouseMode::Move);
}

#[test]
fn state_can_generate_alt_screen_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    let terminfo = TermInfo::from_name("xterm").unwrap();
    vterm
        .write(&CapBuilder::new(&terminfo).cap("smcup").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event: Option<AltScreenEvent> = try_recv_alt_screen_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, true);

    vterm
        .write(&CapBuilder::new(&terminfo).cap("rmcup").build().unwrap())
        .unwrap();
    vterm.flush().unwrap();

    let event: Option<AltScreenEvent> = try_recv_alt_screen_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.is_on, false);
}

#[test]
fn state_can_generate_bell_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 2,
        width: 2,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());

    // BEL - for some reason term crate doesn't know about it?
    vterm.write(b"\x07").unwrap();
    vterm.flush().unwrap();

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_bell_event(&rx);

    assert!(event.is_some());
}

#[test]
fn state_can_generate_resize_events() {
    let mut vterm: VTerm = VTerm::new(&Size {
        height: 5,
        width: 5,
    })
    .unwrap();
    vterm.state_receive_events(&StateCallbacksConfig::all());
    vterm.set_size(&Size::new(2, 3));

    let rx = vterm.state_event_rx.take().unwrap();
    let event = try_recv_resize_event(&rx);

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.size, Size::new(2, 3));
}

// Builds a function that returns a Some of the first event of the given type found on the channel
// or None.
macro_rules! dry {
    ($n:ident, $t:ty, $p:path) => {
        fn $n(rx: &Receiver<StateEvent>) -> Option<$t> {
            while let Ok(e) = rx.try_recv() {
                match e {
                    $p(v) => return Some(v),
                    _ => {}
                }
            }

            None
        }
    };
}

dry!(
    try_recv_alt_screen_event,
    AltScreenEvent,
    StateEvent::AltScreen
);
dry!(try_recv_bell_event, BellEvent, StateEvent::Bell);
dry!(
    try_recv_cursor_blink_event,
    CursorBlinkEvent,
    StateEvent::CursorBlink
);
dry!(
    try_recv_cursor_shape_event,
    CursorShapeEvent,
    StateEvent::CursorShape
);
dry!(
    try_recv_cursor_visible_event,
    CursorVisibleEvent,
    StateEvent::CursorVisible
);
dry!(try_recv_erase_event, EraseEvent, StateEvent::Erase);
dry!(
    try_recv_icon_name_event,
    IconNameEvent,
    StateEvent::IconName
);
dry!(try_recv_init_pen_event, InitPenEvent, StateEvent::InitPen);
//dry!(try_recv_line_info_event, LineInfoEvent, StateEvent::LineInfo);
dry!(try_recv_mouse_event, MouseEvent, StateEvent::Mouse);
dry!(
    try_recv_move_cursor_event,
    MoveCursorEvent,
    StateEvent::MoveCursor
);
dry!(
    try_recv_move_rect_event,
    MoveRectEvent,
    StateEvent::MoveRect
);
dry!(
    try_recv_pen_background_event,
    PenBackgroundEvent,
    StateEvent::PenBackground
);
dry!(
    try_recv_pen_blink_event,
    PenBlinkEvent,
    StateEvent::PenBlink
);
dry!(try_recv_pen_bold_event, PenBoldEvent, StateEvent::PenBold);
dry!(try_recv_pen_font_event, PenFontEvent, StateEvent::PenFont);
dry!(
    try_recv_pen_foreground_event,
    PenForegroundEvent,
    StateEvent::PenForeground
);
dry!(
    try_recv_pen_italic_event,
    PenItalicEvent,
    StateEvent::PenItalic
);
dry!(
    try_recv_pen_reverse_event,
    PenReverseEvent,
    StateEvent::PenReverse
);
dry!(
    try_recv_pen_strike_event,
    PenStrikeEvent,
    StateEvent::PenStrike
);
dry!(
    try_recv_pen_underline_event,
    PenUnderlineEvent,
    StateEvent::PenUnderline
);
dry!(
    try_recv_put_glyph_event,
    PutGlyphEvent,
    StateEvent::PutGlyph
);
dry!(try_recv_resize_event, ResizeEvent, StateEvent::Resize);
dry!(try_recv_reverse_event, ReverseEvent, StateEvent::Reverse);
dry!(
    try_recv_scroll_rect_event,
    ScrollRectEvent,
    StateEvent::ScrollRect
);
dry!(try_recv_title_event, TitleEvent, StateEvent::Title);
