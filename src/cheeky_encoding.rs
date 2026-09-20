//! I'm not certain what this encoding scheme is called. Someone has likely invented it before.
//!
//! Think of encoding as obtaining an *offset* into an `N`-dimensional array, where `cardinalities` represents the tensor shape.
//! Decoding is simply the reverse of that.
//!
//! Some interesting points:
//! - The encode-decode process must be FIFO (I believe).
//! - You can encode into a variable-length integer. This is useful if the data size is variable.
//! - This could be a flexible base for probabilistic compression, such as variable-length encoding.

fn too_big(cardinalities: &[u64]) -> bool {
    let mut product = 1u128;
    for &cardinality in cardinalities {
        product = product.checked_mul(cardinality.into()).unwrap_or(u128::MAX);
    }
    product > u64::MAX as u128 + 1
}

/// # The Rules
///
/// 1. Every value in `data` must be less than the corresponding value in `cardinalities`.
/// 2. The product of all cardinalities must be no larger than `u64::MAX + 1`.
/// 3. A cardinality of zero doesn't make sense.
///
/// Debug assertions will catch rule violations, in release you are on your own.
///
/// # Limitations
///
/// 1. This interface provides no way to store an item with cardinality `u64::MAX + 1` even
///    though rule 2 would allow it. Sure we could probably take `maxes` instead of `cardinalities`.
///    That would also make rule 3 unbreakable.
pub fn encode<const N: usize>(data: &[u64; N], cardinalities: &[u64; N]) -> u64 {
    debug_assert!(!too_big(cardinalities));

    let mut state = 0;
    for (value, cardinality) in data.iter().zip(cardinalities) {
        // for the first element in `data`, the multiply in push is a nop because state is 0
        push(&mut state, *value, *cardinality);
    }
    state
}

pub fn decode<const N: usize>(state: u64, cardinalities: &[u64; N]) -> [u64; N] {
    debug_assert!(!too_big(cardinalities));

    let mut result = [0; N];
    let mut state = state;
    for (i, cardinality) in cardinalities.iter().enumerate().rev() {
        result[i] = pop(&mut state, *cardinality);
    }
    result
}

fn push(state: &mut u64, value: u64, cardinality: u64) {
    debug_assert!(value < cardinality);
    *state *= cardinality;
    *state += value;
}

fn pop(state: &mut u64, cardinality: u64) -> u64 {
    debug_assert!(cardinality > 0);
    let ret = *state % cardinality;
    *state /= cardinality;
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_no_data() {
        let mut state = 0;

        let unit = 0;
        let cardinality = 1; // 1 possible state constitutes no data

        push(&mut state, unit, cardinality);
        assert_eq!(pop(&mut state, cardinality), unit);
    }

    #[test]
    fn store_one_bit() {
        let cardinality = 2;

        for bit in [0, 1] {
            let mut state = 0;
            push(&mut state, bit, cardinality);
            assert_eq!(pop(&mut state, cardinality), bit);
        }
    }

    #[test]
    fn store_more_than_one_but_less_than_two_bits() {
        let cardinality = 3; // {0, 1, 2}

        for value in [0, 1, 2] {
            let mut state = 0;
            push(&mut state, value, cardinality);
            assert_eq!(pop(&mut state, cardinality), value);
        }
    }

    #[test]
    fn store_one_bit_twice() {
        let cardinality = 2;

        for (a, b) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let mut state = 0;
            push(&mut state, a, cardinality);
            push(&mut state, b, cardinality);
            assert_eq!(pop(&mut state, cardinality), b);
            assert_eq!(pop(&mut state, cardinality), a);
        }
    }

    #[test]
    fn store_list() {
        fn check<const N: usize>(data_vs_cardinalities: &[(u64, u64); N]) {
            let just_data = data_vs_cardinalities.map(|(data, _)| data);
            let just_possibilities = data_vs_cardinalities.map(|(_, possibilities)| possibilities);

            let state = encode(&just_data, &just_possibilities);
            let decoded = decode(state, &just_possibilities);
            assert_eq!(decoded, just_data);
        }

        check(&[
            (0, 1), // nothing, zero bits of data being stored
            (0, 2), // one bit
            (1, 2),
            (0, 3), // log2(3) ~= 1.585 bits
            (1, 3),
            (2, 3),
            (0, 4), // log2(4) = 2 bits
            (1, 4),
            (2, 4),
            (3, 4),
            (0, 5), // log2(5) ~= 2.321 bits
            (1, 5),
            (2, 5),
            (3, 5),
            (4, 5),
        ]);
        check(&[(1, 2); 32]);
        check(&[(2, 3); 40]); // 64 / 1.585 ~= 40.4
        check(&[(4, 5); 27]); // 64 / 2.321 ~= 27.5
        check(&[(u64::MAX - 1, u64::MAX)]);
        check(&[(u8::MAX.into(), Into::<u64>::into(u8::MAX) + 1); 8]);
        check(&[(u16::MAX.into(), Into::<u64>::into(u16::MAX) + 1); 4]);
        check(&[(u32::MAX.into(), Into::<u64>::into(u32::MAX) + 1); 2]);
    }
}
