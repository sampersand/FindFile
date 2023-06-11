fn byte_to_hex(byte: u8) -> Option<u8> {
	// TODO: update this when I get back on wifi
	match byte {
		b'0'..=b'9' => Some(byte - b'0'),
		b'a'..=b'f' => Some((byte - b'a') + 10),
		b'A'..=b'F' => Some((byte - b'A') + 10),
		_ => None,
	}
}

// note the `\` should have been parsed ahead of time
pub fn parse_string_escape(inp: &[u8]) -> Option<(char, &[u8])> {
	match inp.get(0)? {
		byte @ (b'\\' | b'\'' | b'\"' | b'{' | b'}' | b' ') => Some((*byte as char, &inp[1..])),
		b'n' => Some(('\n', &inp[1..])),
		b't' => Some(('\t', &inp[1..])),
		b'r' => Some(('\r', &inp[1..])),
		b'f' => Some(('\x0C', &inp[1..])),
		b'0' => Some(('\0', &inp[1..])),
		b'x' => {
			let upper = inp.get(1).copied().and_then(byte_to_hex)?;
			let lower = inp.get(2).copied().and_then(byte_to_hex)?;
			let chr = ((upper << 4) | lower) as char;

			Some((chr, &inp[3..]))
		}
		b'u' => {
			let chr = char::from_u32(
				((inp.get(1).copied().and_then(byte_to_hex)? as u32) << 12)
					| ((inp.get(2).copied().and_then(byte_to_hex)? as u32) << 8)
					| ((inp.get(3).copied().and_then(byte_to_hex)? as u32) << 4)
					| ((inp.get(4).copied().and_then(byte_to_hex)? as u32) << 0),
			)?;
			Some((chr, &inp[5..]))
		}
		b'U' => todo!("support `\\U`?"),
		_ => None,
	}
}
