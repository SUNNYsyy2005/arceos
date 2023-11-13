use pc_keyboard::{
    DecodedKey, HandleControl, Keyboard, ScancodeSet1,
};
use pc_keyboard::layouts::Us104Key;
use x86_64::instructions::port::Port;

use spinlock::SpinNoIrq;


#[cfg(feature = "irq")]
use crate::irq;

static KEYBUFFER: SpinNoIrq<KeyboardBuffer> = SpinNoIrq::new(KeyboardBuffer::new());
static KEYBOARD: SpinNoIrq<Keyboard<Us104Key, ScancodeSet1>> = SpinNoIrq::new(
    Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore),
);
/// 键盘中断号
const KEYBOARD_IRQ: usize = 0x21;
/// 键盘输出缓存端口
const PORT_KB_DATA: u16 = 0x60;
/// buffer大小
const BUF_SIZE: usize = 100;


/// 键盘输入buffer 环形队列
struct KeyboardBuffer {
    buf: [u8; BUF_SIZE],
    head_index: usize,
    tail_index: usize,
    count: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            head_index: 0,
            tail_index: 0,
            count: 0,
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.tail_index >= self.buf.len() {
            self.tail_index = 0;
        }
        let res = if self.count > 0 {
            let res = Some(self.buf[self.tail_index]);
            self.tail_index += 1;
            self.count -= 1;
            res
        } else {
            None
        };
        return res;
    }
    fn push(&mut self, data: u8) {
        if self.head_index >= self.buf.len() {
            self.head_index = 0;
        }
        self.buf[self.head_index] = data;
        self.head_index += 1;
        self.count += 1;
    }
}

//在中断处理函数中完成对扫描码的转换然后写入buffer。
fn handler() {
    let mut keybuffer = KEYBUFFER.lock();
    let scancode: u8 = unsafe { Port::new(PORT_KB_DATA).read()};
    keybuffer.push(scancode);
    /* let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                
                DecodedKey::Unicode(character) => keybuffer.push(character as u8),
                _ => {}
            }
        }
    } */
}
#[derive(Copy,Clone)]
struct KeyCode { 
    ascii1: u8,// no shift code
    ascii2: u8,// shift code
    scode: u8,// scan code
    kcode: u8,// key code
}
#[derive(Copy,Clone)]
enum KeyState { 
    KeyRelease = 0,
    KeyPress = 0x1000000,
}
//键盘映射表 扫描码->键码
static KEY_MAP: [KeyCode; 94] = [
/* 0x00 - none*/	KeyCode{ ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x01-ESC*/	KeyCode{ascii1:0, ascii2:0, scode: 0x01,kcode:0x1B},
/*0x02-'1'*/	KeyCode{ascii1:'1' as u8 , ascii2:'!' as u8 , scode: 0x02,kcode:0x31},
/*0x03-'2'*/	KeyCode{ascii1:'2' as u8 , ascii2:'@' as u8 , scode: 0x03,kcode:0x32},
/*0x04-'3'*/	KeyCode{ascii1:'3' as u8 , ascii2:'#' as u8 , scode: 0x04,kcode:0x33},
/*0x05-'4'*/	KeyCode{ascii1:'4' as u8 , ascii2:'$' as u8 , scode: 0x05,kcode:0x34},
/*0x06-'5'*/	KeyCode{ascii1:'5' as u8 , ascii2:'%' as u8 , scode: 0x06,kcode:0x35},
/*0x07-'6'*/	KeyCode{ascii1:'6' as u8 , ascii2:'^' as u8 , scode: 0x07,kcode:0x36},
/*0x08-'7'*/	KeyCode{ascii1:'7' as u8 , ascii2:'&' as u8 , scode: 0x08,kcode:0x37},
/*0x09-'8'*/	KeyCode{ascii1:'8' as u8 , ascii2:'*' as u8 , scode: 0x09,kcode:0x38},
/*0x0A-'9'*/	KeyCode{ascii1:'9' as u8 , ascii2:'(' as u8 , scode: 0x0A,kcode:0x39},
/*0x0B-'0'*/	KeyCode{ascii1:'0' as u8 , ascii2:')' as u8 , scode: 0x0B,kcode:0x30},
/*0x0C-'-'*/	KeyCode{ascii1:'-' as u8 , ascii2:'_' as u8 , scode: 0x0C,kcode:0xBD},
/*0x0D-'='*/	KeyCode{ascii1:'=' as u8 , ascii2:'+' as u8 , scode: 0x0D,kcode:0xBB},
/*0x0E-BS*/	KeyCode{ascii1:0, ascii2:0, scode: 0x0E,kcode:0x08},
/*0x0F-TAB*/	KeyCode{ascii1:0, ascii2:0, scode: 0x0F,kcode:0x09},
/*0x10-'q'*/	KeyCode{ascii1:'q' as u8 , ascii2:'Q' as u8 , scode: 0x10,kcode:0x51},
/*0x11-'w'*/	KeyCode{ascii1:'w' as u8 , ascii2:'W' as u8 , scode: 0x11,kcode:0x57},
/*0x12-'e'*/	KeyCode{ascii1:'e' as u8 , ascii2:'E' as u8 , scode: 0x12,kcode:0x45},
/*0x13-'r'*/	KeyCode{ascii1:'r' as u8 , ascii2:'R' as u8 , scode: 0x13,kcode:0x52},
/*0x14-'t'*/	KeyCode{ascii1:'t' as u8 , ascii2:'T' as u8 , scode: 0x14,kcode:0x54},
/*0x15-'y'*/	KeyCode{ascii1:'y' as u8 , ascii2:'Y' as u8 , scode: 0x15,kcode:0x59},
/*0x16-'u'*/	KeyCode{ascii1:'u' as u8 , ascii2:'U' as u8 , scode: 0x16,kcode:0x55},
/*0x17-'i'*/	KeyCode{ascii1:'i' as u8 , ascii2:'I' as u8 , scode: 0x17,kcode:0x49},
/*0x18-'o'*/	KeyCode{ascii1:'o' as u8 , ascii2:'O' as u8 , scode: 0x18,kcode:0x4F},
/*0x19-'p'*/	KeyCode{ascii1:'p' as u8 , ascii2:'P' as u8 , scode: 0x19,kcode:0x50},
/*0x1A-'['*/	KeyCode{ascii1:'[' as u8 ,ascii2:'{' as u8 , scode: 0x1A,kcode:0xDB},
/*0x1B-']'*/	KeyCode{ascii1:']' as u8 ,ascii2:'}' as u8 , scode: 0x1B,kcode:0xDD},
/*0x1C-CR/LF*/	KeyCode{ascii1:0, ascii2:0, scode: 0x1C,kcode:0x0D},
/*0x1D-l.Ctrl*/	KeyCode{ascii1:0, ascii2:0, scode: 0x1D,kcode:0x11},
/*0x1E-'a'*/	KeyCode{ascii1:'a' as u8 , ascii2:'A' as u8 , scode: 0x1E,kcode:0x41},
/*0x1F-'s'*/	KeyCode{ascii1:'s' as u8 , ascii2:'S' as u8 , scode: 0x1F,kcode:0x53},
/*0x20-'d'*/	KeyCode{ascii1:'d' as u8 , ascii2:'D' as u8 , scode: 0x20,kcode:0x44},
/*0x21-'f'*/	KeyCode{ascii1:'f' as u8 , ascii2:'F' as u8 , scode: 0x21,kcode:0x46},
/*0x22-'g'*/	KeyCode{ascii1:'g' as u8 , ascii2:'G' as u8 , scode: 0x22,kcode:0x47},
/*0x23-'h'*/	KeyCode{ascii1:'h' as u8 , ascii2:'H' as u8 , scode: 0x23,kcode:0x48},
/*0x24-'j'*/	KeyCode{ascii1:'j' as u8 , ascii2:'J' as u8 , scode: 0x24,kcode:0x4A},
/*0x25-'k'*/	KeyCode{ascii1:'k' as u8 , ascii2:'K' as u8 , scode: 0x25,kcode:0x4B},
/*0x26-'l'*/	KeyCode{ascii1:'l' as u8 , ascii2:'L' as u8 , scode: 0x26,kcode:0x4C},
/*0x27-';'*/	KeyCode{ascii1:';' as u8 , ascii2:':' as u8 , scode: 0x27,kcode:0xBA},
/*0x28-'\''*/	KeyCode{ascii1:'\'' as u8 , ascii2:'\"' as u8 , scode: 0x28,kcode:0xDE},
/*0x29-'`'*/	KeyCode{ascii1:'`' as u8 , ascii2:'~' as u8 , scode: 0x29,kcode:0xC0},
/*0x2A-l.SHIFT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x2A,kcode:0x10},
/*0x2B-'\'*/	KeyCode{ascii1:'\\' as u8 , ascii2:'|' as u8 , scode: 0x2B,kcode:0xDC},
/*0x2C-'z'*/	KeyCode{ascii1:'z' as u8 , ascii2:'Z' as u8 , scode: 0x2C,kcode:0x5A},
/*0x2D-'x'*/	KeyCode{ascii1:'x' as u8 , ascii2:'X' as u8 , scode: 0x2D,kcode:0x58},
/*0x2E-'c'*/	KeyCode{ascii1:'c' as u8 , ascii2:'C' as u8 , scode: 0x2E,kcode:0x43},
/*0x2F-'v'*/	KeyCode{ascii1:'v' as u8 , ascii2:'V' as u8 , scode: 0x2F,kcode:0x56},
/*0x30-'b'*/	KeyCode{ascii1:'b' as u8 , ascii2:'B' as u8 , scode: 0x30,kcode:0x42},
/*0x31-'n'*/	KeyCode{ascii1:'n' as u8 , ascii2:'N' as u8 , scode: 0x31,kcode:0x4E},
/*0x32-'m'*/	KeyCode{ascii1:'m' as u8 , ascii2:'M' as u8 , scode: 0x32,kcode:0x4D},
/*0x33-' as u8 ,'*/	KeyCode{ascii1:',' as u8 , ascii2:'<' as u8 , scode: 0x33,kcode:0xBC},
/*0x34-'.'*/	KeyCode{ascii1:'.' as u8 , ascii2:'>' as u8 , scode: 0x34,kcode:0xBE},
/*0x35-'/'*/	KeyCode{ascii1:'/' as u8 , ascii2:'?' as u8 , scode: 0x35,kcode:0xBF},
/*0x36-r.SHIFT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x36,kcode:0x10},
/*0x37-'*'*/	KeyCode{ascii1:'*' as u8 , ascii2:'*' as u8 , scode: 0x37,kcode:0x6A},
/*0x38-ALT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x38,kcode:0x12},
/*0x39-''*/	KeyCode{ascii1:' ' as u8 ,ascii2:0, scode: 0x39,kcode:0x20},
/*0x3A-CapsLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3A,kcode:0x14},
/*0x3B-F1*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3B,kcode:0x70},
/*0x3C-F2*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3C,kcode:0x71},
/*0x3D-F3*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3D,kcode:0x72},
/*0x3E-F4*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3E,kcode:0x73},
/*0x3F-F5*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3F,kcode:0x74},
/*0x40-F6*/	KeyCode{ascii1:0, ascii2:0, scode: 0x40,kcode:0x75},
/*0x41-F7*/	KeyCode{ascii1:0, ascii2:0, scode: 0x41,kcode:0x76},
/*0x42-F8*/	KeyCode{ascii1:0, ascii2:0, scode: 0x42,kcode:0x77},
/*0x43-F9*/	KeyCode{ascii1:0, ascii2:0, scode: 0x43,kcode:0x78},
/*0x44-F10*/	KeyCode{ascii1:0, ascii2:0, scode: 0x44,kcode:0x79},
/*0x45-NumLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x45,kcode:0x90},
/*0x46-ScrLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x46,kcode:0x91},
/*0x47-Home*/	KeyCode{ascii1:0, ascii2:0, scode: 0x47,kcode:0x24},
/*0x48-Up*/	KeyCode{ascii1:0, ascii2:0, scode: 0x48,kcode:0x26},
/*0x49-PgUp*/	KeyCode{ascii1:0, ascii2:0, scode: 0x49,kcode:0x21},
/*0x4A-'-'*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4A,kcode:0x6D},
/*0x4B-Left*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4B,kcode:0x25},
/*0x4C-MID*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4C,kcode:0x0C},
/*0x4D-Right*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4D,kcode:0x27},
/*0x4E-'+'*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4E,kcode:0x6B},
/*0x4F-End*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4F,kcode:0x23},
/*0x50-Down*/	KeyCode{ascii1:0, ascii2:0, scode: 0x50,kcode:0x28},
/*0x51-PgDown*/	KeyCode{ascii1:0, ascii2:0, scode: 0x51,kcode:0x22},
/*0x52-Insert*/	KeyCode{ascii1:0, ascii2:0, scode: 0x52,kcode:0x2D},
/*0x53-Del*/	KeyCode{ascii1:0, ascii2:0, scode: 0x53,kcode:0x2E},
/*0x54-Enter*/	KeyCode{ascii1:0, ascii2:0, scode: 0x54,kcode:0x0D},
/*0x55-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x56-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x57-F11*/	KeyCode{ascii1:0, ascii2:0, scode: 0x57,kcode:0x7A},
/*0x58-F12*/	KeyCode{ascii1:0, ascii2:0, scode: 0x58,kcode:0x7B},
/*0x59-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x5A-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x5B-LeftWin*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5B,kcode:0x5B},
/*0x5C-RightWin*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5C,kcode:0x5C},
/*0x5D-Apps*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5D,kcode:0x5D}
];
fn is_shift(sc: u8) -> bool {
    sc == 0x2A || sc == 0x36
}

fn is_caps_lock(sc: u8) -> bool {
    sc == 0x3A
}

fn is_num_lock(sc: u8) -> bool {
    sc == 0x45
}
//判断是否为字母，从而判断是否需要大写
fn is_letter(sc:u8)->bool{
    if sc>=0x10&&sc<=0x19 {
        return true;
    }else if sc>=0x1E&&sc<=0x26 {
        return true;
    }else if sc>=0x2C&&sc<=0x32 {
        return true;
    }
    return false;
}
fn make_code(pkc: &KeyCode, shift: i32, caps_lock: i32, num_lock: i32, e0: i32) -> u32 {
   // println!("\n{} {} {} {} {}", pkc.ascii1 as char, shift, caps_lock, num_lock, e0);
    0
}

fn key_type(sc: u8) -> u32 {
    if sc & 0x80 != 0 {
        KeyState::KeyRelease as u32
    } else {
        KeyState::KeyPress as u32
    }
}
//处理大写锁定、上档键
fn pause_handler(src: u8) -> u32 {
    let mut sc=src;
    let pressed = key_type(sc);
    if pressed == 0 {
        sc -= 0x80;
    }
    unsafe{
        if is_shift(sc) {
            SHIFT = pressed;
            return 1;
        } else if is_caps_lock(sc) {
            if pressed==0{
                CAPS_LOCK = 1 - CAPS_LOCK;
            }
            return 1;
        } else if is_num_lock(sc) {
            if pressed==0{
                NUM_LOCK = 1 - NUM_LOCK;
            }
            return 1;
        }
    }
    
    0
}
fn get(sc:u8)->Option<KeyCode>{
    for i in 0..94{
        if KEY_MAP[i].scode == sc{
            return Some(KEY_MAP[i]);
        }
    }
    None
}
static mut SHIFT: u32 = 0;
static mut CAPS_LOCK: u32 = 0;
static mut NUM_LOCK: u32 = 0;
//检测是否完成按键过程(某键是否被释放)
fn key_handler(src: u8) -> u32 {
    let mut sc=src;
    static mut E0: u32 = 0;

    let mut ret = 0;

    unsafe {
        if sc == 0xE0 {
            E0 = 1;
            ret = 1;
        } else {
            let pressed = key_type(sc);
            let mut pkc = None;
            if pressed == 0 {
                sc -= 0x80;
            }
            pkc = get(sc as u8);
            if let Some(key_code) = pkc {
                if key_code.scode != 0 {
                    ret = 1;
                    let mut code = 0;
                    code = pressed | make_code(&key_code, SHIFT as i32, CAPS_LOCK as i32, NUM_LOCK as i32, E0 as i32);
                    E0 = 0;
                    if pressed == 0 {
                        return 1;
                    }
                    else{
                        return 3;
                    }
                    
                }
            }
        }
    }

    ret
}
//键盘终端处理函数
fn put_scan_code() {
    let sc: u8 = unsafe { Port::new(PORT_KB_DATA).read()};
    if pause_handler(sc) != 0 {
        // Pause Key 处理大写锁定、NumLock、Shift等不用输出的字符
    } else if key_handler(sc) != 0 {
        if key_handler(sc) == 1{
            let mut keybuffer = KEYBUFFER.lock();
            let pkc = get((sc-0x80) as u8).unwrap();
            unsafe{
                if SHIFT == 0 {
                    if CAPS_LOCK !=0 {
                        if is_letter(sc-0x80) {
                           // keybuffer.push('M' as u8);
                            keybuffer.push(pkc.ascii2 as u8);
                        } else {
                            keybuffer.push(pkc.ascii1 as u8);
                        }
                    }else {
                        //keybuffer.push('N' as u8);
                        keybuffer.push(pkc.ascii1 as u8);
                    }
                } else {
                    keybuffer.push(pkc.ascii2 as u8);
                }
            }
            
        }
        // Normal Key
    } else {
        // Unknown Key
    }
}

fn fetch_key_code() -> u32 {
    0
}
//注册键盘中断处理函数。
pub(super) fn init() {
    #[cfg(feature = "irq")]
    {
        irq::register_handler(KEYBOARD_IRQ, put_scan_code);
    //irq::register_handler(KEYBOARD_IRQ, handler);
    }
}

pub fn getchar() -> Option<u8> {
    KEYBUFFER.lock().pop()
}
