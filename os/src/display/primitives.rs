
macro_rules! impl_primitive_print {
    ($typ: ty, $n_cap: expr, $h_cap: expr) => {
        impl PrimitiveDisplay<$n_cap, $h_cap> for $typ {
            fn as_numeric_ascii(&self) -> LenArray<u8, $n_cap> {
                let mut value: [u8; $n_cap] = [0; $n_cap];
                let mut len = 0;
                let mut accounted = 0;
                for i in (0..$n_cap).rev() {
                    let mut mult = 1;
                    for _ in 0..i {
                        mult *= 10;
                    }
                    let this_char = (self - accounted) / mult;
                    if this_char > 0 {
                        accounted += this_char * mult;
                    } else {
                        continue;
                    }
                    if accounted > 0 {
                        value[len] = (this_char as u8) + 0x30;
                        len += 1;
                    }
                }
                LenArray(len, value)
            }
            fn as_hexadecimal_ascii(&self) -> LenArray<u8, $h_cap> {
                let mut value: [u8; $h_cap] = [0; $h_cap];
                for i in 0..$h_cap {
                    let mul = (i << 2);
                    let this_char = (self & (15 << mul)) >> mul;
                    value[{$h_cap-1}-i] = HEX_INDICES[this_char as usize];
                }
                LenArray($h_cap, value)
            }
        }
    }
}

struct LenArray<T, const CAP: usize>(usize, [T; CAP]);
impl<T, const CAP: usize> AsRef<[T]> for LenArray<T, CAP> {
    fn as_ref(&self) -> &[T] {
        &self.1[..self.0]
    }
}
trait PrimitiveDisplay<const LEN_CAP: usize, const HEX_LEN: usize> {
    fn as_numeric_ascii(&self) -> LenArray<u8, LEN_CAP>;
    fn as_hexadecimal_ascii(&self) -> LenArray<u8, HEX_LEN>;
}

const HEX_INDICES: &[u8] = &[b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];

impl_primitive_print!(u8, 3, 2);
impl_primitive_print!(u16, 5, 4);
impl_primitive_print!(u32, 10, 8);
impl_primitive_print!(u64, 20, 16);