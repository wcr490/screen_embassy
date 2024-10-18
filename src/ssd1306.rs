use core::ptr;

use once_cell::sync::Lazy;

use crate::{command_raw, Error, ScreenCommand, ScreenI2c};

static INIT_SSD1306: Lazy<[u16; 44]>  = Lazy::new(|| {
[
  17,                                                             // number of initializers
  command_raw!(SSD1306_DISPLAY_OFF), 0,                                         // 0xAE = Set Display OFF
  command_raw!(SSD1306_SET_MUX_RATIO), 1, 0x1F,                                 // 0xA8 - 0x3F for 128 x 64 version (64MUX)
                                                                  //      - 0x1F for 128 x 32 version (32MUX)
  command_raw!(SSD1306_MEMORY_ADDR_MODE), 1, 0x00,                              // 0x20 = Set Memory Addressing Mode
                                                                  // 0x00 - Horizontal Addressing Mode
                                                                  // 0x01 - Vertical Addressing Mode
                                                                  // 0x02 - Page Addressing Mode (RESET)
  command_raw!(SSD1306_SET_START_LINE), 0,                                      // 0x40
  command_raw!(SSD1306_DISPLAY_OFFSET), 1, 0x00,                                // 0xD3
  command_raw!(SSD1306_SEG_REMAP_OP), 0,                                        // 0xA0 / remap 0xA1
  command_raw!(SSD1306_COM_SCAN_DIR_OP), 0,                                     // 0xC0 / remap 0xC8
  command_raw!(SSD1306_COM_PIN_CONF), 1, 0x02,                                  // 0xDA, 0x12 - Disable COM Left/Right remap, Alternative COM pin configuration
                                                                  //       0x12 - for 128 x 64 version
                                                                  //       0x02 - for 128 x 32 version
  command_raw!(SSD1306_SET_CONTRAST), 1, 0x7F,                                  // 0x81, 0x7F - reset value (max 0xFF)
  command_raw!(SSD1306_DIS_ENT_DISP_ON), 0,                                     // 0xA4
  command_raw!(SSD1306_DIS_NORMAL), 0,                                          // 0xA6
  command_raw!(SSD1306_SET_OSC_FREQ), 1, 0x80,                                  // 0xD5, 0x80 => D=1; DCLK = Fosc / D <=> DCLK = Fosc
  command_raw!(SSD1306_SET_PRECHARGE), 1, 0xc2,                                 // 0xD9, higher value less blinking
                                                                  // 0xC2, 1st phase = 2 DCLK,  2nd phase = 13 DCLK
  command_raw!(SSD1306_VCOM_DESELECT), 1, 0x20,                                 // Set V COMH Deselect, reset value 0x22 = 0,77xUcc
 command_raw!( SSD1306_SET_CHAR_REG), 1, 0x14,                                  // 0x8D, Enable charge pump during display on
  command_raw!(SSD1306_DEACT_SCROLL), 0,                                        // 0x2E
  command_raw!(SSD1306_DISPLAY_ON), 0                                           // 0xAF = Set Display ON
]
});



#[allow(unused, non_camel_case_types)]
#[warn(private_interfaces)]
pub(crate) enum Command {
    /*
    All of that are from https://github.com/Matiasus/SSD1306/blob/master/lib/ssd1306.h
    Thanks
     */
    SSD1306_COMMAND,          // Continuation bit=1, D/C=0; 1000 0000
    SSD1306_COMMAND_STREAM,   // Continuation bit=0, D/C=0; 0000 0000
    SSD1306_DATA,             // Continuation bit=1, D/C=1; 1100 0000
    SSD1306_DATA_STREAM,      // Continuation bit=0, D/C=1; 0100 0000
    SSD1306_SET_MUX_RATIO,    // Set MUX ratio to N+1 MUX, N=A[5:0] : from 16MUX to 64MUX
    SSD1306_DISPLAY_OFFSET,   // Set Display Offset
    SSD1306_DISPLAY_ON,       // Display ON in normal mode
    SSD1306_DISPLAY_OFF,      // Display OFF (sleep mode)
    SSD1306_DIS_ENT_DISP_ON,  // Entire Display ON, Output ignores RAM content
    SSD1306_DIS_IGNORE_RAM,   // Resume to RAM content display, Output follows RAM content
    SSD1306_DIS_NORMAL, // Normal display, 0 in RAM: OFF in display panel, 1 in RAM: ON in display panel
    SSD1306_DIS_INVERSE, // Inverse display, 0 in RAM: ON in display panel, 1 in RAM: OFF in display panel
    SSD1306_DEACT_SCROLL, // Stop scrolling that is configured by command 26h/27h/29h/2Ah
    SSD1306_ACTIVE_SCROLL, // Start scrolling that is configured by the scrolling setup commands:26h/27h/29h/2Ah
    SSD1306_SET_START_LINE, // Set Display Start Line
    SSD1306_MEMORY_ADDR_MODE, // Set Memory, Addressing Mode
    SSD1306_SET_COLUMN_ADDR, // Set Column Address
    SSD1306_SET_PAGE_ADDR, // Set Page Address
    SSD1306_SEG_REMAP,     // Set Segment Re-map, X[0]=0b column address 0 is mapped to SEG0
    SSD1306_SEG_REMAP_OP,  // Set Segment Re-map, X[0]=1b: column address 127 is mapped to SEG0
    SSD1306_COM_SCAN_DIR, // Set COM Output, X[3]=0b: normal mode (RESET) Scan from COM0 to COM[N –1], e N is the Multiplex ratio
    SSD1306_COM_SCAN_DIR_OP, // Set COM Output, X[3]=1b: remapped mode. Scan from COM[N-1] to COM0, e N is the Multiplex ratio
    SSD1306_COM_PIN_CONF,    // Set COM Pins Hardware Configuration,
    // A[4]=0b, Sequential COM pin configuration, A[4]=1b(RESET), Alternative COM pin configuration
    // A[5]=0b(RESET), Disable COM Left/Right remap, A[5]=1b, Enable COM Left/Right remap
    SSD1306_SET_CONTRAST, // Set Contrast Control, Double byte command to select 1 to 256 contrast steps, increases as the value increases
    SSD1306_SET_OSC_FREQ, // Set Display Clock Divide Ratio/Oscillator Frequency
    // A[3:0] : Define the divide ratio (D) of the  display clocks (DCLK): Divide ratio= A[3:0] + 1, RESET is 0000b (divide ratio = 1)
    // A[7:4] : Set the Oscillator Frequency, FOSC. Oscillator Frequency increases with the value of A[7:4] and vice versa. RESET is 1000b
    SSD1306_SET_CHAR_REG, // Charge Pump Setting, A[2] = 0b, Disable charge pump(RESET), A[2] = 1b, Enable charge pump during display on
    // The Charge Pump must be enabled by the following command:
    // 8Dh ; Charge Pump Setting
    // 14h ; Enable Charge Pump
    // AFh; Display ON
    SSD1306_SET_PRECHARGE, // Set Pre-charge Period
    SSD1306_VCOM_DESELECT, // Set VCOMH Deselect Leve
    SSD1306_NOP,           // No operation
    SSD1306_RESET,         // Maybe SW RESET, @source https://github.com/SmingHub/Sming/issues/501
}


impl ScreenCommand for Command {
    fn raw(&self) -> u16 {
        match self {
            Command::SSD1306_COMMAND => 0x80,
            Command::SSD1306_COMMAND_STREAM => 0x00,
            Command::SSD1306_DATA => 0xC0,
            Command::SSD1306_DATA_STREAM => 0x40,
            Command::SSD1306_SET_MUX_RATIO => 0xA8,
            Command::SSD1306_DISPLAY_OFFSET => 0xD3,
            Command::SSD1306_DISPLAY_ON => 0xAF,
            Command::SSD1306_DISPLAY_OFF => 0xAE,
            Command::SSD1306_DIS_ENT_DISP_ON => 0xA4,
            Command::SSD1306_DIS_IGNORE_RAM => 0xA5,
            Command::SSD1306_DIS_NORMAL => 0xA6,
            Command::SSD1306_DIS_INVERSE => 0xA7,
            Command::SSD1306_DEACT_SCROLL => 0x2E,
            Command::SSD1306_ACTIVE_SCROLL => 0x2F,
            Command::SSD1306_SET_START_LINE => 0x40,
            Command::SSD1306_MEMORY_ADDR_MODE => 0x20,
            Command::SSD1306_SET_COLUMN_ADDR => 0x21,
            Command::SSD1306_SET_PAGE_ADDR => 0x22,
            Command::SSD1306_SEG_REMAP => 0xA0,
            Command::SSD1306_SEG_REMAP_OP => 0xA1,
            Command::SSD1306_COM_SCAN_DIR => 0xC0,
            Command::SSD1306_COM_SCAN_DIR_OP => 0xC8,
            Command::SSD1306_COM_PIN_CONF => 0xDA,
            Command::SSD1306_SET_CONTRAST => 0x81,
            Command::SSD1306_SET_OSC_FREQ => 0xD5,
            Command::SSD1306_SET_CHAR_REG => 0x8D,
            Command::SSD1306_SET_PRECHARGE => 0xD9,
            Command::SSD1306_VCOM_DESELECT => 0xDB,
            Command::SSD1306_NOP => 0xE3,
            Command::SSD1306_RESET => 0xE4,
        }
    }
}


// TODO: refactor by bare pointer
#[allow(clippy::large_enum_variant)]
pub enum BufferSize {
    SSD1306_128x64([u8; 128 * 8]),
    SSD1306_128x32([u8; 128 * 4]),
}
impl BufferSize {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize{
        match self {
            BufferSize::SSD1306_128x64(buf) => buf.len(),
            BufferSize::SSD1306_128x32(buf) => buf.len(),
        }
    }
    pub fn byte(&self, index: usize) -> Option<u8> {
        match self {
            BufferSize::SSD1306_128x64(buf) => {
                if index < 128*8 - 1 {
                    return Some(buf[index]);
                }
                None
            },
            BufferSize::SSD1306_128x32(buf) => {
                if index < 128*4 - 1 {
                    return Some(buf[index]);
                }
                None
            },
        }
    }
    pub fn clean(&mut self) {
        match self {
            BufferSize::SSD1306_128x64(buf) => unsafe {
                ptr::write_bytes(buf, 0, buf.len());
            },
            BufferSize::SSD1306_128x32(buf) => unsafe {
                ptr::write_bytes(buf, 0, buf.len());
            }
        }
    }
}

pub struct Ssd1306<T> 
where T: embedded_hal_async::i2c::I2c
{
    bus: ScreenI2c<T>,
    addr: u8,
    buf: BufferSize,
    anchor: (u8, u8),
}

impl<T> Ssd1306<T>
where T: embedded_hal_async::i2c::I2c 
{
    pub fn new(bus: T) -> Self {
        Ssd1306 {
            bus: ScreenI2c::new(bus),
            addr: 0x3c,
            buf: BufferSize::SSD1306_128x32([0u8; 128 * 4]),
            anchor: (0, 0),
        }
    }
    pub async fn init(&mut self, address: u8) -> Result<(), Error<T::Error>> {
        self.addr = address;
        let mut index = 0;
        let init_command = &*INIT_SSD1306;
        let mut left_commands_num = init_command[index];
        index += 1;
        while left_commands_num > 0 {
            let raw_command = init_command[index];
            index += 1;
            let mut left_arguments_num = init_command[index];
            self.send_raw_command(raw_command).await?;
            while left_arguments_num > 0 {
                let arguement = init_command[index];
                index += 1;
                self.send_raw_command(arguement).await?;
                left_arguments_num -= 1;
            }
            left_commands_num -= 1;
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////
    ///      Private 
    //////////////////////////////////////////////////////
    pub(crate) async fn send_command(&mut self, command: Command) -> Result<(), Error<T::Error>> {
        self.bus.write_command(self.addr, command).await
    }
    pub(crate) async fn send_raw_command(&mut self, command: u16) -> Result<(), Error<T::Error>> {
        self.bus.write_raw_command(self.addr, command).await
    }
    pub(crate) async fn send_byte_data(&mut self, byte: u8) -> Result<(), Error<T::Error>> {
        self.bus.write_byte(self.addr, byte).await
    }
    pub(crate) async fn check_position(&self, x: u8, y: u8) -> Result<(), Error<()>> {
        if let BufferSize::SSD1306_128x32(_) = self.buf {
            if x >= 128 || y >= 32 {
                return Err(Error::Range(()));
            }
        }
        if let BufferSize::SSD1306_128x64(_) = self.buf {
            if x >= 128 || y >= 64 {
                return Err(Error::Range(()));
            }
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////
    ///      Command
    //////////////////////////////////////////////////////
    pub async fn normal_screen(&mut self) -> Result<(), Error<T::Error>> {
        self.send_command(Command::SSD1306_DIS_NORMAL).await
    }
    pub async fn inverse_screen(&mut self) -> Result<(), Error<T::Error>> {
        self.send_command(Command::SSD1306_DIS_INVERSE).await
    }
    pub async fn update_screen(&mut self) -> Result<(), Error<T::Error>> {
        self.send_command(Command::SSD1306_DATA_STREAM).await?;
        let len = self.buf.len();
        let mut index = 0;
        while index < len {
            if let Some(byte) = self.buf.byte(index) {
                self.send_byte_data(byte).await?;
                index += 1;
            }
            else {
                return Err(Error::Range(()));
            }
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////
    ///      Buffer
    //////////////////////////////////////////////////////
    pub async fn clear_screen(&mut self) {
        self.buf.clean();
    }
    pub async fn set_position(&mut self, x: u8, y: u8) {
        self.anchor = (x, y);
    }
    pub async fn draw_pixel(&mut self, x: u8, y: u8) -> Result<(), Error<()>>{
        self.anchor = (x, y);
        self.check_position(x, y).await?;
        let page = y >> 3;
        let index_in_page = y - (page << 3);
        let x_index: usize = (128*page + x) as usize;
        if let BufferSize::SSD1306_128x32(mut buf) = self.buf {
            buf[x_index] |= 1 << index_in_page;
        }
        if let BufferSize::SSD1306_128x64(mut buf) = self.buf {
            buf[x_index] |= 1 << index_in_page;
        }
        Ok(())
    }
}