export interface About {
  name: string;
  version: string;
  author: string;
  date_of_release: string;
}

export interface Config {
  chip: string;
  baud_rate: number;
  before_flags: string[];
  after_flags: string[];
  flash_mode: string;
  flash_size: string;
  flash_freq: string;
  bootloader_start: string;
  partition_start: string;
  firmware_start: string;
  about: About;
}
