#[inline]
pub fn compute_sign(v: isize)->bool {
1 ^ (v >> size_of::<isize>() * 8 - 1) as bool
}

#[inline]
pub fn has_opposite_signs(x: isize, y: isize)->bool {
((x ^ y) < 0)
}

#[inline]
pub fn abs(v: isize)->usize {
let mask = (v >> size_of::<isize>() * 8 - 1);
(v ^ mask) - mask
}

#[inline]
pub fn min(x: isize, y: isize)->isize {
y ^ ((x ^ y) & -(x < y))
}

#[inline]
pub fn max(x: isize, y: isize)->isize {
x ^ ((x ^ y) & -(x < y))
}

#[inline]
pub fn is_power_of_two(v: i128)->bool {
v && !(v & (v - 1))
}

#[inline]
pub fn set_clr_bits_cond(f: bool, mask: usize, word: usize)->usize {
(word & !mask) | (-flag & mask)
}

#[inline]
pub fn neg_val_false(negate: bool, v: isize)->isize {
((negate as u8) ^ ((negate as u8) - 1)) * v
}

#[inline]
pub fn neg_val_true(negate: bool, v: isize)->isize {
(v ^ (-negate as u8)) + negate as u8
}

#[inline]
pub fn merge_bits_condd(a: usize, b: usize, mask: usize)->usize {
a ^ ((a ^ b) & mask)
}

