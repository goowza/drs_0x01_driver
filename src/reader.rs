#![allow(dead_code)]
#![allow(unused_imports)]

use arrayvec::ArrayVec;

use addr::WritableEEPAddr;
use addr::WritableRamAddr;
use addr::EEPReadData;
use addr::RamReadData;

pub const TRAME_READER_INTERNAL_BUFFER_SIZE: usize = 64;

// Structure renvoyée en fin de machine

pub struct ACKPacket {
    psize : u8,
    pid : u8,
    cmd : Command,
    chk1 : u8,
    chk2 : u8,
    data : AssociatedData,
    error : StatusError,
    detail : StatusDetail,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Command {
    EEPWrite{detail : StatusDetail, error : StatusError},
    EEPRead{data : EEPReadData, detail : StatusDetail, error : StatusError},
    RamWrite{detail : StatusDetail, error : StatusError},
    RamRead{data : RamReadData, detail : StatusDetail, error : StatusError},
    IJog{detail : StatusDetail, error : StatusError},
    SJog{detail : StatusDetail, error : StatusError},
    Stat{detail : StatusDetail, error : StatusError},
    Rollback{detail : StatusDetail, error : StatusError},
    Reboot{detail : StatusDetail, error : StatusError},
    Nothing,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InternalCommand {
    EEPWrite,
    EEPRead,
    RamWrite,
    RamRead,
    IJog,
    SJog,
    Stat,
    Rollback,
    Reboot,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InternalCommandWithData {
    EEPWrite,
    EEPRead{data : EEPReadData},
    RamWrite,
    RamRead{data : RamReadData},
    IJog,
    SJog,
    Stat,
    Rollback,
    Reboot,
    Nothing,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatusError {
    ExceedInputVoltageLimit,
    ExceedAllowedPOTLimit,
    ExceedTemperatureLimit,
    InvalidPacket,
    OverloadDetected,
    DriverFaultDetected,
    EEPREGDistorded,
    NoError,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatusDetail {
    MovingFlag,
    ImpositionFlag,
    ChecksumError,
    UnknownCommand,
    ExceedREGRange,
    GarbageDetected,
    MotorOnFlag,
    NoDetail,
}

#[derive(Copy,Clone,Debug)]
enum AssociatedData {
    EEP(EEPReadData),
    Ram(RamReadData),
    Nothing
}

/*#[derive(Debug, PartialEq)]
struct ACKPacket {
    command: Command,
    data_addr: Option<u8>,
    data_len: Option<u8>,
    data: Option<[u8; 16]>, // doc p20
    error: Option<StatusError>,
    detail: Option<StatusDetail>,
}*/

struct ACKReader {
    pub(crate) state: ReaderState,
    buffer: ArrayVec<[ACKPacket; TRAME_READER_INTERNAL_BUFFER_SIZE]>,
}

// Structure permettant de gérer la machine à états
#[derive(Debug, Clone, Copy)]
pub(crate) enum ReaderState {
    H1,
    H2,
    Psize,
    Pid {
        size : u8
    },
    Cmd {
        size : u8,
        pid : u8,
    },
    Checksum1 {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
    },
    Checksum2 {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
    },
    DataAddr {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
        chk2 : u8,
    },
    DataLenEEP {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
        chk2 : u8,
        data : EEPReadData,
    },
    Data1EEP {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
        chk2 : u8,
        data : EEPReadData,
    },
    Data2EEP {
        size : u8,
        pid : u8,
        cmd: InternalCommandWithData,
        chk1 : u8,
        chk2 : u8,
        data : EEPReadData,
    },
    DataLenRAM {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
        chk2 : u8,
        data : RamReadData,
    },
    Data1RAM {
        size : u8,
        pid : u8,
        cmd: InternalCommand,
        chk1 : u8,
        chk2 : u8,
        data : RamReadData,
    },
    Data2RAM {
        size : u8,
        pid : u8,
        cmd: InternalCommandWithData,
        chk1 : u8,
        chk2 : u8,
        data : RamReadData,
    },
    Error {
        size : u8,
        pid : u8,
        cmd: InternalCommandWithData,
        chk1 : u8,
        chk2 : u8,
        payload : AssociatedData,
    },
    Detail {
        size : u8,
        pid : u8,
        cmd: InternalCommandWithData,
        chk1 : u8,
        chk2 : u8,
        payload : AssociatedData,
        status_error : StatusError,
    },
    SendToBuffer {
        size : u8,
        pid : u8,
        cmd: InternalCommandWithData,
        chk1 : u8,
        chk2 : u8,
        payload : AssociatedData,
        status_error : StatusError,
        status_detail : StatusDetail,
    }

}

impl InternalCommand {
    fn into_command_with_data(&self) -> InternalCommandWithData {
        match *self {
            EEPWrite => InternalCommandWithData::EEPWrite,
            RamWrite => InternalCommandWithData::RamWrite,
            IJog => InternalCommandWithData::IJog,
            SJog => InternalCommandWithData::SJog,
            Stat => InternalCommandWithData::Stat,
            Rollback => InternalCommandWithData::Rollback,
            Reboot => InternalCommandWithData::Reboot,
            _ => InternalCommandWithData::Nothing,
        }
    }
}

impl InternalCommandWithData {
    fn into_command(&self, error : StatusError, detail : StatusDetail) -> Command {
        match *self {
            EEPWrite => Command::EEPWrite {error,detail},
            //EEPRead{data} => Command::EEPRead {data,error,detail},
            RamWrite => Command::RamWrite {error,detail},
            //RamRead{data} => Command::RamRead {data,error,detail},
            IJog => Command::IJog {error,detail},
            SJog => Command::SJog {error,detail},
            Rollback => Command::Rollback {error,detail},
            Reboot => Command::Reboot {error,detail},
            _ => Command::Nothing,
        }
    }
}

impl ReaderState {

    fn step(&mut self, byte : u8) -> Option<Command> {
        use reader::ReaderState::*;
        use reader::InternalCommand::*;
        use reader::AssociatedData::*;
        use reader::StatusError::*;
        use reader::StatusDetail::*;
        use addr::WritableRamAddr::*;
        use addr::WritableEEPAddr::*;
        use addr::ReadableEEPAddr;
        use addr::ReadableRamAddr;
        use addr::RamReadData;
        use addr::EEPReadData;

        let a = match *self {
            H1 => {
                *self = H2
            },
            H2 => {
                *self = Psize
            },
            Psize => {
                *self = Pid {
                    size: byte
                }
            },
            Pid { size } => {
                *self = Cmd {
                    size: size,
                    pid: byte,
                }
            },
            Cmd { size, pid } => {
                let mut command: InternalCommand;
                match byte {
                    0x41 => command = EEPWrite,
                    0x42 => command = EEPRead,
                    0x43 => command = RamWrite,
                    0x44 => command = RamRead,
                    0x45 => command = IJog,
                    0x46 => command = SJog,
                    0x47 => command = Stat,
                    0x48 => command = Rollback,
                    0x49 => command = Reboot,
                    _ => *self = H1,
                }
                *self = Checksum1 {
                    size: size,
                    pid: pid,
                    cmd: command
                }
            }
            Checksum1 { size, pid, cmd } => {
                *self = Checksum2 {
                    size: size,
                    pid: pid,
                    cmd: cmd,
                    chk1: byte,
                }
            },
            Checksum2 { size, pid, cmd, chk1 } if (cmd == EEPRead || cmd == RamRead) => {
                *self = DataAddr {
                    size: size,
                    pid: pid,
                    cmd: cmd,
                    chk1: chk1,
                    chk2: byte,
                }
            },
            Checksum2 { size, pid, cmd, chk1 } => {
                *self = Error {
                    size: size,
                    pid: pid,
                    cmd: cmd.into_command_with_data(),
                    chk1: chk1,
                    chk2: byte,
                    payload: Nothing,
                }
            },
            DataAddr { size, pid, cmd, chk1, chk2 } => {
                match cmd {
                    EEPRead => {
                        *self = match ReadableEEPAddr::try_from(byte) {
                            Ok(data_addr) => DataLenEEP {
                                size: size,
                                pid: pid,
                                cmd: cmd,
                                chk1: chk1,
                                chk2: chk2,
                                data : EEPReadData{
                                    addr : data_addr,
                                    data_len : 0,
                                    data : [0,0],
                                },
                            },
                            Err(_) => H1
                        }
                    },
                    RamRead => {
                        *self = match ReadableRamAddr::try_from(byte) {
                            Ok(data_addr) => DataLenRAM {
                                size: size,
                                pid: pid,
                                cmd: cmd,
                                chk1: chk1,
                                chk2: chk2,
                                data : RamReadData{
                                    addr : data_addr,
                                    data_len : 0,
                                    data : [0,0],
                                },
                            },
                            Err(_) => H1
                        }
                    },
                }
            },
            DataLenEEP { size, pid, cmd, chk1, chk2, data } => {
                let new_data = EEPReadData {
                    addr : data.addr,
                    data_len : byte,
                    data : [0,0],
                };
                *self = Data1EEP {
                    size: size,
                    pid: pid,
                    cmd: cmd,
                    chk1: chk1,
                    chk2: chk2,
                    data : new_data,
                };
            },
            DataLenRAM { size, pid, cmd, chk1, chk2, data } => {
                let new_data = RamReadData {
                    addr : data.addr,
                    data_len : byte,
                    data : [0,0],
                };
                *self = Data1RAM {
                    size: size,
                    pid: pid,
                    cmd: cmd,
                    chk1: chk1,
                    chk2: chk2,
                    data: new_data,
                }
            }
            Data1EEP {size, pid, cmd, chk1, chk2, data} => {
                let new_data = EEPReadData {
                    addr : data.addr,
                    data_len : data.data_len,
                    data : [byte,0],
                };
                *self = Data2EEP {
                    size: size,
                    pid: pid,
                    cmd : InternalCommandWithData::EEPRead {
                        data : new_data,
                    },
                    chk1: chk1,
                    chk2: chk2,
                    data : new_data,
                }
            }
            Data2EEP {size, pid, cmd, chk1, chk2, data} => {
                let new_data = EEPReadData {
                    addr : data.addr,
                    data_len : data.data_len,
                    data : [data.data[0],byte],
                };
                *self = Error {
                    size: size,
                    pid: pid,
                    cmd : cmd,
                    chk1: chk1,
                    chk2: chk2,
                    payload : AssociatedData::EEP(new_data),
                }
            }
            Data1RAM {size, pid, cmd, chk1, chk2, data} => {
                let new_data = RamReadData {
                    addr : data.addr,
                    data_len : data.data_len,
                    data : [byte,0],
                };
                *self = Data2RAM {
                    size: size,
                    pid: pid,
                    cmd : InternalCommandWithData::RamRead {
                        data : new_data,
                    },
                    chk1: chk1,
                    chk2: chk2,
                    data : new_data,
                }
            }
            Data2RAM {size, pid, cmd, chk1, chk2, data} => {
                let new_data = RamReadData {
                    addr : data.addr,
                    data_len : data.data_len,
                    data : [data.data[0],byte],
                };
                *self = Error {
                    size: size,
                    pid: pid,
                    cmd : cmd,
                    chk1: chk1,
                    chk2: chk2,
                    payload : AssociatedData::Ram(new_data),
                }
            }
            Error {size, pid, cmd, chk1, chk2, payload} => {
                match byte {
                    0x00 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: NoError,
                    },
                    0x01 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: ExceedInputVoltageLimit,
                    },
                    0x02 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: ExceedAllowedPOTLimit,
                    },
                    0x04 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: ExceedTemperatureLimit,
                    },
                    0x08 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: InvalidPacket,
                    },
                    0x10 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: OverloadDetected,
                    },
                    0x20 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: DriverFaultDetected,
                    },
                    0x40 => *self = Detail {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error: EEPREGDistorded,
                    },
                    _ => *self = H1,
                }
            }
            Detail {size, pid, cmd, chk1, chk2, payload, status_error} => {
                match byte {
                    0x00 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : NoDetail,
                    },
                    0x01 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : MovingFlag,
                    },
                    0x02 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : ImpositionFlag,
                    },
                    0x04 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : ChecksumError,
                    },
                    0x08 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : UnknownCommand,
                    },
                    0x10 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : ExceedREGRange,
                    },
                    0x20 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : GarbageDetected,
                    },
                    0x40 => *self = SendToBuffer {
                        size: size,
                        pid: pid,
                        cmd: cmd,
                        chk1: chk1,
                        chk2: chk2,
                        payload: payload,
                        status_error : status_error,
                        status_detail : MotorOnFlag,
                    },
                    _ => *self = H1,
                }
            },
            SendToBuffer {size, pid, cmd, chk1, chk2, payload, status_error, status_detail} => {
                let packet = ACKPacket {
                    psize : size,
                    pid : pid,
                    cmd : cmd.into_command(status_error,status_detail),
                    chk1 : chk1,
                    chk2 : chk2,
                    data : payload,
                    error : status_error,
                    detail : status_detail,
                };
                // renvoyer ACKPacket qq part ? ._.
            },
            _ => (),
        };
    }
}

impl ACKReader {
    // Creation d'un ACKReader a l'état H1 et avec un buffer vide
    pub fn new() -> ACKReader {
        ACKReader {
            state: ReaderState::H1,
            buffer: ArrayVec::new(),
        }
    }

    // Renvoi le premier ACKPacket du buffer
    pub fn pop_ack(&mut self) -> Option<ACKPacket> {
        self.buffer.pop()
    }

    // Renvoi la taille du buffer
    pub fn get_buffer_size(&mut self) -> usize {
        self.buffer.len()
    }

    // Lit les octetc de l'ACK un par un
    fn parse(&mut self, buf: &[u8]) {
        for byte in buf {
            self.step(*byte);
        }
    }

    pub fn step(&mut self, byte : u8) {
    }

    // Lit un octet et fait avancer ou non l'état
    /*pub fn step(&mut self, byte: u8) {
        use reader::ReaderState::*;
        use reader::Command::*;
        use reader::StatusDetail::*;
        use reader::StatusError::*;

        match self.state.clone() {
            H1 if byte == 0xFF => self.state = H2,
            H2 if byte == 0xFF => self.state = Psize,
            Psize => self.state = Pid,
            Pid => self.state = Cmd,
            Cmd => match byte {
                0x41 => self.state = Checksum1 { cmd: EEPWrite },
                0x42 => self.state = Checksum1 { cmd: EEPRead },
                0x43 => self.state = Checksum1 { cmd: RamWrite },
                0x44 => self.state = Checksum1 { cmd: RamRead },
                0x45 => self.state = Checksum1 { cmd: IJog },
                0x46 => self.state = Checksum1 { cmd: SJog },
                0x47 => self.state = Checksum1 { cmd: Stat },
                0x48 => self.state = Checksum1 { cmd: Rollback },
                0x49 => self.state = Checksum1 { cmd: Reboot },
                _ => {
                    self.state = H1;
                }
            },
            Checksum1 { ref cmd } => {
                self.state = Checksum2 { cmd: *cmd };
            }
            Checksum2 { ref cmd } => {
                self.state = DataAddr { cmd: *cmd };
            }
            // Si la commande etait EEPRead ou RamRead, on recupere des donnees
            DataAddr { ref cmd } if (*cmd == EEPRead || *cmd == RamRead) => {
                self.state = DataLen {
                    cmd: *cmd,
                    data_addr: Some(byte),
                };
            }
            // Sinon on passe à l'état suivant
            DataAddr { ref cmd } => {
                self.state = Error {
                    cmd: *cmd,
                    data_addr: None,
                    data_len: None,
                    data: None,
                };
            }
            // Si on doit recuperer des donnees, on renvoie aussi la taille de ces donnees
            DataLen {
                ref cmd,
                ref data_addr,
            }
                if byte > 0 =>
            {
                self.state = Data {
                    cmd: *cmd,
                    data_addr: *data_addr,
                    data_len: Some(byte),
                    data: Some([0x00; 16]),
                    current_index: 0,
                };
            }
            // Si DataLen = 0 passer a l'etat suivant
            DataLen {
                ref cmd,
                ref data_addr,
            }
                if byte == 0 =>
            {
                self.state = Error {
                    cmd: *cmd,
                    data_addr: *data_addr,
                    data_len: None,
                    data: None,
                };
            }
            Data {
                ref cmd,
                ref data_addr,
                ref data_len,
                data,
                current_index,
            }
                if current_index < data_len.unwrap() - 1 =>
            {
                let mut in_data = data.unwrap(); // c'est pas joli mais ca marche :)
                in_data[current_index as usize] = byte; // c'est pas joli mais ca marche :)
                self.state = Data {
                    cmd: *cmd,
                    data_addr: *data_addr,
                    data_len: *data_len,
                    data: Some(in_data),
                    current_index: current_index + 1,
                }
            }
            Data {
                ref cmd,
                ref data_addr,
                ref data_len,
                ref data,
                current_index,
            }
                if current_index == data_len.unwrap() - 1 =>
            {
                let mut in_data = data.unwrap(); // c'est pas joli mais ca marche :)
                in_data[current_index as usize] = byte; // c'est pas joli mais ca marche :)
                self.state = Error {
                    cmd: *cmd,
                    data_addr: *data_addr,
                    data_len: *data_len,
                    data: Some(in_data),
                };
            }
            Error {
                ref cmd,
                ref data_addr,
                ref data_len,
                ref mut data,
            } => match byte {
                0x00 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: None,
                    }
                }
                0x01 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(ExceedInputVoltageLimit),
                    }
                }
                0x02 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(ExceedAllowedPOTLimit),
                    }
                }
                0x04 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(ExceedTemperatureLimit),
                    }
                }
                0x08 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(InvalidPacket),
                    }
                }
                0x10 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(OverloadDetected),
                    }
                }
                0x20 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(DriverFaultDetected),
                    }
                }
                0x40 => {
                    self.state = Detail {
                        cmd: *cmd,
                        data_addr: *data_addr,
                        data_len: *data_len,
                        data: *data,
                        error: Some(EEPREGDistorded),
                    }
                }
                _ => self.state = H1,
            },
            Detail {
                ref cmd,
                ref data_addr,
                ref data_len,
                ref data,
                ref error,
            } => {
                let mut detail: Option<StatusDetail>;
                match byte {
                    0x00 => detail = None,
                    0x01 => detail = Some(MovingFlag),
                    0x02 => detail = Some(ImpositionFlag),
                    0x04 => detail = Some(ChecksumError),
                    0x08 => detail = Some(UnknownCommand),
                    0x10 => detail = Some(ExceedREGRange),
                    0x20 => detail = Some(GarbageDetected),
                    0x40 => detail = Some(MotorOnFlag),
                    _ => detail = None,
                }
                let packet = ACKPacket {
                    command: *cmd,
                    data_addr: *data_addr,
                    data_len: *data_len,
                    data: *data,
                    error: *error,
                    detail: detail,
                };
                self.buffer.push(packet);
                self.state = H1;
            }
            _ => self.state = H1,
        }
    } */
}

#[cfg(test)]
mod test {
    use reader::{ACKReader, Command, StatusDetail, StatusError};
    #[test]
    fn test() {
        /*let mut reader = ACKReader::new();

        // Test de EEPRead
        let packet_eepread = [
            0xFF, 0xFF, 0x0F, 0xFD, 0x42, 0x4C, 0xB2, 0x1E, 0x04, 0xB8, 0x01, 0x40, 0x1F, 0x08,
            0x20,
        ];

        reader.parse(&packet_eepread);

        let data_eepread: [u8; 16] = [
            0xB8, 0x01, 0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        assert_eq!(
            reader.pop_ack(),
            Some(ACKPacket {
                command: Command::EEPRead,
                data_addr: Some(0x1E),
                data_len: Some(0x04),
                data: Some(data_eepread),
                error: Some(StatusError::InvalidPacket),
                detail: Some(StatusDetail::GarbageDetected),
            })
        );

        // Test de RAMRead
        let packet_ramread = [
            0xFF, 0xFF, 0x0C, 0xFD, 0x44, 0xC2, 0x3C, 0x35, 0x01, 0x01, 0x00, 0x40,
        ];

        reader.parse(&packet_ramread);

        let data_ramread: [u8; 16] = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        assert_eq!(
            reader.pop_ack(),
            Some(ACKPacket {
                command: Command::RamRead,
                data_addr: Some(0x35),
                data_len: Some(0x01),
                data: Some(data_ramread),
                error: None,
                detail: Some(StatusDetail::MotorOnFlag),
            })
        );*/
    }
}
