use crate::{Coordinate, Rectangle};

pub struct Hilbert {
    env: Rectangle,
    x_min: f64,
    y_min: f64,
    x_scale: f64,
    y_scale: f64,
}

impl Hilbert {
    #[allow(clippy::float_cmp)]
    pub fn new(env: Rectangle) -> Self {
        if env.is_empty() {
            return Hilbert {
                env,
                x_scale: 0.,
                y_scale: 0.,
                x_min: 0.,
                y_min: 0.,
            };
        }
        let x_scale;
        let y_scale;
        let x_min;
        let y_min;
        let hilbert_max = ((1 << 16) - 1) as f64;
        if env.x_min == env.x_max {
            x_scale = 0.;
            x_min = 0.;
        } else {
            x_scale = hilbert_max / (env.x_max - env.x_min);
            x_min = env.x_min;
        }
        if env.y_min == env.y_max {
            y_scale = 0.;
            y_min = 0.;
        } else {
            y_scale = hilbert_max / (env.y_max - env.y_min);
            y_min = env.y_min;
        }
        Hilbert {
            env,
            x_min,
            y_min,
            x_scale,
            y_scale,
        }
    }

    /**
     * Like hilbert, but checks that position is not None and with range.
     *
     * None positions and those out of range are assigned maxint.
     */
    #[allow(dead_code)]
    pub fn safe_hilbert(&self, position: Option<Coordinate>) -> u32 {
        match position {
            Some(p) if self.env.contains(p) => self.hilbert(p),
            _ => u32::max_value(),
        }
    }

    /**
     * Returns the hilbert index of position in the rectangle.
     *
     * This does not check bounds; it will probably panic for positions
     * outside of the rectangle.  This behavior should not be relied on.
     */
    pub fn hilbert(&self, position: Coordinate) -> u32 {
        let x = self.x_scale * (position.x - self.x_min);
        let y = self.y_scale * (position.y - self.y_min);
        Self::hilbert_normalized(x.floor() as u32, y.floor() as u32)
    }

    /**
     * Fast Hilbert curve algorithm by http://threadlocalmutex.com/
     * Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
     */
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    pub fn hilbert_normalized(x: u32, y: u32) -> u32 {
        let mut a = x ^ y;
        let mut b = 0xFFFF ^ a;
        let mut c = 0xFFFF ^ (x | y);
        let mut d = x & (y ^ 0xFFFF);

        let mut A = a | (b >> 1);
        let mut B = (a >> 1) ^ a;
        let mut C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
        let mut D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

        a = A;
        b = B;
        c = C;
        d = D;
        A = (a & (a >> 2)) ^ (b & (b >> 2));
        B = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
        C ^= (a & (c >> 2)) ^ (b & (d >> 2));
        D ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

        a = A;
        b = B;
        c = C;
        d = D;
        A = (a & (a >> 4)) ^ (b & (b >> 4));
        B = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
        C ^= (a & (c >> 4)) ^ (b & (d >> 4));
        D ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

        a = A;
        b = B;
        c = C;
        d = D;
        C ^= (a & (c >> 8)) ^ (b & (d >> 8));
        D ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

        a = C ^ (C >> 1);
        b = D ^ (D >> 1);

        let mut i0 = x ^ y;
        let mut i1 = b | (0xFFFF ^ (i0 | a));

        i0 = (i0 | (i0 << 8)) & 0x00FF_00FF;
        i0 = (i0 | (i0 << 4)) & 0x0F0F_0F0F;
        i0 = (i0 | (i0 << 2)) & 0x3333_3333;
        i0 = (i0 | (i0 << 1)) & 0x5555_5555;

        i1 = (i1 | (i1 << 8)) & 0x00FF_00FF;
        i1 = (i1 | (i1 << 4)) & 0x0F0F_0F0F;
        i1 = (i1 | (i1 << 2)) & 0x3333_3333;
        i1 = (i1 | (i1 << 1)) & 0x5555_5555;

        (i1 << 1) | i0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized() {
        let h = Hilbert::hilbert_normalized(12345, 67890);
        assert_eq!(h, 99_289_669);
    }

    #[test]
    fn hilbert_from_position() {
        let total_rect = Rectangle::new((1., 2.).into(), (2., 8.).into());
        let position = Coordinate::new(1.25, 5.);
        let h = Hilbert::new(total_rect);
        let result = h.hilbert(position);
        // x = floor(0.25 * 65535) y = floor(0.5 * 65535)
        // or hilbert_normalized(16383, 32767)
        assert_eq!(result, 805_306_368);
    }

    #[test]
    fn hilbert_from_none_position() {
        let total_rect = Rectangle::new((1., 2.).into(), (2., 3.).into());
        let position = None;
        let h = Hilbert::new(total_rect);
        let result = h.safe_hilbert(position);
        assert_eq!(result, u32::max_value());
    }

    #[test]
    fn hilbert_from_out_of_bounds_position() {
        let total_rect = Rectangle::new((1., 2.).into(), (2., 3.).into());
        let position = Some(Coordinate::new(4., 4.));
        let h = Hilbert::new(total_rect);
        let result = h.safe_hilbert(position);
        assert_eq!(result, u32::max_value());
    }

    #[test]
    fn hilbert_with_degenerate_rect() {
        let position = Coordinate::new(1., 1.);
        let total_rect = Rectangle::new(position, position);
        let h = Hilbert::new(total_rect);
        let result = h.hilbert(position);
        assert_eq!(result, 0);
    }

    #[test]
    fn hilbert_ordering() {
        let total_rect = Rectangle::new((0., 0.).into(), (4., 4.).into());
        let h = Hilbert::new(total_rect);
        let hi0 = h.hilbert(Coordinate::new(0., 0.));
        let hi1 = h.hilbert(Coordinate::new(1., 1.));
        let hi2 = h.hilbert(Coordinate::new(1., 3.));
        let hi3 = h.hilbert(Coordinate::new(3., 3.));
        let hi4 = h.hilbert(Coordinate::new(3., 1.));
        assert!(hi0 < hi1);
        assert!(hi1 < hi2);
        assert!(hi2 < hi3);
        assert!(hi3 < hi4);
    }
}
