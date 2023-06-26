use num_traits::PrimInt;

pub fn get_num_of_bits<T: PrimInt>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub fn get_highest_bit<T: PrimInt>(v: T) -> Option<usize> {
    let nb = get_num_of_bits::<T>();
    let lz = v.leading_zeros() as usize;
    ( lz != nb ).then(|| nb - (lz + 1))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_num_of_bits() {
        assert_eq!(get_num_of_bits::<u8>(), 8);
        assert_eq!(get_num_of_bits::<u16>(), 16);
        assert_eq!(get_num_of_bits::<u32>(), 32);
        assert_eq!(get_num_of_bits::<u64>(), 64);
        assert_eq!(get_num_of_bits::<u128>(), 128);
    }

    #[test]
    fn test_highest_bit() {
        assert_eq!(get_highest_bit::<u8>(0), None);
        assert_eq!(get_highest_bit::<u8>(0x10), Some(4));
        assert_eq!(get_highest_bit::<u8>(128), Some(7));
        assert_eq!(get_highest_bit::<u16>(0xffff), Some(15));
    }
}
