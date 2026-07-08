Name of the backing type.

Backing type used for internal manipulations.

For example, we have a 4-byte array and a value with an 8-bit width that starts at bit 4 (first byte).
This means our value overlap two bytes.
To read the 8-bit value, we need to read first 2 bytes of the array. Two bytes is a 16-bit value (u16, i16);
This 2-byte value is our backing type.

#### Visual example:

8-bit value (u8):   `[0000_0011]`
4-bytes array: `[0000_0000]` `[0011_0000]` `[0000_0000]` `[0000_0000]`

1) read first 2 bytes (u16):   `[0000_0000_0011_0000]`
2) shift to start:             `[0000_0000_0000_0011]`
3) then cast to our [repr] type: `[0000_0011]`
