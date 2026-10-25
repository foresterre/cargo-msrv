#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    AtMost(usize),
    Exactly(usize),
}

impl Width {
    pub fn total(self) -> usize {
        match self {
            Width::AtMost(width) | Width::Exactly(width) => width,
        }
    }

    /// Adjust the column widths, i.e.:
    /// * Shrink the column if it is too wide
    /// * Grow the column if it is too narrow
    pub fn adjust_column_widths(
        self,
        mut widths: Vec<usize>,
        minimums: &[usize],
        available: usize,
    ) -> Vec<usize> {
        shrink(&mut widths, minimums, available);

        if let Width::Exactly(_) = self {
            grow(&mut widths, available);
        }

        widths
    }
}

fn shrink(widths: &mut [usize], minimums: &[usize], available: usize) {
    while widths.iter().sum::<usize>() > available {
        let widest = (0..widths.len())
            .filter(|&column| widths[column] > minimums[column])
            .max_by_key(|&column| widths[column]);

        match widest {
            Some(column) => widths[column] -= 1,
            None => break,
        }
    }
}

fn grow(widths: &mut [usize], available: usize) {
    let total = widths.iter().sum::<usize>();
    let columns = widths.len();

    if columns == 0 || total >= available {
        return;
    }

    let surplus = available - total;

    for (column, width) in widths.iter_mut().enumerate() {
        *width += surplus / columns + usize::from(column < surplus % columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_widest_column_first() {
        let widths = Width::AtMost(0).adjust_column_widths(vec![4, 10, 10], &[1, 1, 1], 20);

        assert_eq!(widths, vec![4, 8, 8]);
    }

    #[test]
    fn keeps_columns_at_their_minimum() {
        let widths = Width::AtMost(0).adjust_column_widths(vec![6, 10], &[6, 1], 4);

        assert_eq!(widths, vec![6, 1]);
    }

    #[test]
    fn spreads_surplus_when_exact() {
        let widths = Width::Exactly(0).adjust_column_widths(vec![2, 2, 2], &[1, 1, 1], 10);

        assert_eq!(widths, vec![4, 3, 3]);
    }
}
