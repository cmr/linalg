use std::vec;
use std::fmt;
use std::num::{Zero, One, one};

/// A two-dimensional matrix.
#[deriving(Clone)]
pub struct Mat2<T> {
    // INVARIANT: data.len() == n, data.iter().all(|v| v.len() == m)
    // If this gets violated, shit hits the fan ASAP.
    priv data: ~[~[T]],
    /// Number of rows
    priv n: uint,
    /// Number of columns
    priv m: uint,
}

pub struct RowIterator<'a, T> {
    priv mat: &'a Mat2<T>,
    priv i: uint,
}

impl<'a, T> Iterator<&'a [T]> for RowIterator<'a, T> {
    fn next(&mut self) -> Option<&'a [T]> {
        let r = self.mat.get_row_opt(self.i);
        // handle overflow
        if r.is_some() { self.i += 1; }
        r
    }
}

pub struct ColumnIterator<'a, T> {
    priv mat: &'a Mat2<T>,
    priv col: uint,
    priv i: uint,
}

impl<'a, T> Iterator<&'a T> for ColumnIterator<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        let r = match self.mat.data.get_opt(self.i) {
            Some(s) => s.get_opt(self.col),
            None => None
        };

        // handle overflow
        if r.is_some() { self.i += 1; }
        r
    }
}

// TODO: remove clone bound?
impl<T: Default+Clone> Mat2<T> {
    /// Create a new (n x m) matrix, using the Default implementation of T
    pub fn new(n: uint, m: uint) -> Mat2<T> {
        let data = vec::from_elem(n, vec::from_elem(m, Default::default()));

        Mat2 { data: data, n: n, m: m }
    }
}

impl<T: fmt::Default> fmt::Default for Mat2<T> {
    fn fmt(s: &Mat2<T>, f: &mut fmt::Formatter) {
        let b = &mut f.buf;
        b.write(bytes!("[\n"));
        for row in s.row_iter() {
            for it in row.iter() {
                write!(*b, "{} ", *it);
            }
            b.write(bytes!("\n"));
        }
        b.write(bytes!("]\n"));
    }
}

impl<T> Mat2<T> {
    /// Create a new (n x m) matrix, using `f` to create each element. `f` is given the coordinate
    /// (row, column) for each element it's constructing.
    pub fn new_with(n: uint, m: uint, f: |uint, uint| -> T) -> Mat2<T> {
        let data = vec::from_fn(n, |n| vec::from_fn(m, |m| f(n,m)));

        Mat2 { data: data, n: n, m: m }
    }

    /// Create a new matrix from a vector. Returns None if the inner vectors don't all have the same
    /// length, or if the vector is empty.
    pub fn from_vec(m: ~[~[T]]) -> Option<Mat2<T>> {
        let n = m.len();

        if n == 0 {
            return None;
        }

        let l = m[0].len();

        if m.iter().all(|x| x.len() == l) {
            Some(Mat2 { data: m, n: n, m: l })
        } else {
            None
        }
    }

    /// Return the dimensions of the matrix, (m, n)
    pub fn get_dimension(&self) -> (uint, uint) {
        (self.m, self.n)
    }

    /// Swap two rows. Fails if either of the indices are out of bounds.
    pub fn swap_rows(&mut self, i: uint, j: uint) {
        self.data.swap(i, j);
    }

    /// Set a row to the given vector. Fails if `i` is out of bounds.
    pub fn set_row(&mut self, i: uint, r: ~[T]) {
        self.data[i] = r;
    }

    /// Get the row at `i` as a slice. Fails if `i` is out of bounds.
    pub fn get_row<'a>(&'a self, i: uint) -> &'a [T] {
        self.data[i].as_slice()
    }

    /// Get a reference to the element at row `i`, column `j` (both starting at 0). Returns `None`
    /// if `i` or `j` are out of bounds.
    pub fn get_opt<'a>(&'a self, i: uint, j: uint) -> Option<&'a T> {
        if (i > self.m || j > self.n) {
            None
        } else {
            Some(&self.data[i][j])
        }
    }

    /// Get a reference to the element at row `i`, column `j` (both starting at 0). Fails if `i` or
    /// `j` are out of bounds.
    pub fn get<'a>(&'a self, i: uint, j: uint) -> &'a T {
        &self.data[i][j]
    }

    /// Get the row at `i` as a slice. Returns `None` if `i` is out of bounds.
    pub fn get_row_opt<'a>(&'a self, i: uint) -> Option<&'a [T]> {
        self.data.get_opt(i).map(|o| o.as_slice())
    }

    /// Append a column to the matrix. Returns true if the insert succeeded, false otherwise.
    pub fn append_column(&mut self, column: ~[T]) -> bool {
        // this makes sure the unsafe_mut_ref below will be valid
        if self.n != column.len() { return false; }

        self.n += 1;

        for (idx, itm) in column.move_iter().enumerate() {
            unsafe { (*self.data.unsafe_mut_ref(idx)).push(itm); }
        }

        true
    }

    /// Append a row to the matrix. Returns true if the insert succeeded, false otherwise.
    pub fn append_row(&mut self, row: ~[T]) -> bool {
        if self.m != row.len() { return false; }

        self.n += 1;

        self.data.push(row);

        true
    }

    /// ["Augment"](http://goo.gl/Q1hIuC) a matrix with this one. Takes the columns of `other` and
    /// appends them to this matrix. Returns true if the augment succeeded, false otherwise.
    pub fn augment(&mut self, other: Mat2<T>) -> bool {
        if self.n != other.n { return false; }
        self.m += other.m;

        for (idx, row) in other.data.move_iter().enumerate() {
            unsafe  { (*self.data.unsafe_mut_ref(idx)).push_all_move(row); }
        }

        true
    }

    /// Iterate over the rows of a matrix.
    pub fn row_iter<'a>(&'a self) -> RowIterator<'a, T> {
        RowIterator {
            mat: self,
            i: 0
        }
    }

    /// Iterate over the items column `col` (0-based) of a matrix. This does *NOT* iterate over all
    /// columns.  If you want that, transpose the matrix and `row_iter` over that (it requires the
    /// same amount of work).
    pub fn column_iter<'a>(&'a self, col: uint) -> ColumnIterator<'a, T> {
        ColumnIterator {
            mat: self,
            col: col,
            i: 0
        }
    }
}

impl<T: Mul<T, T>> Mat2<T> {
    /// Scale a row by a scalar.
    pub fn scale_row(&mut self, i: uint, a: T) {
        for idx in range(0, self.data[i].len()) {
            self.data[i][idx] = self.data[i][idx] * a;
        }
    }
}

impl<T: Eq> Eq for Mat2<T> {
    fn eq(&self, other: &Mat2<T>) -> bool {
        self.data == other.data
    }
}

impl<T: Mul<T, T> + Add<T, T> + Clone> Mat2<T> {
    /// Add a row `i` scaled by `a` to another row `j`. Fails if either of the indices are out of
    /// bounds.
    pub fn add_scaled(&mut self, i: uint, j: uint, a: T) {
        let r = self.data[i].iter().enumerate().map(|(i, x)| x.clone() * a + self.data[j][i])
                    .to_owned_vec();
        self.set_row(j, r);
    }
}

impl<T: fmt::Default+Mul<T, T> + Add<T, T> + Div<T, T> + Zero + One + Eq + Clone> Mat2<T> {
    /// Do Gauss-Jordan elimination on this matrix to convert it into Reduced Row-Echelon Form.
    pub fn reduce(&mut self) {
        // translation of pseudocode at http://linear.ups.edu/html/section-RREF.html
        let (m, n, mut r) = (self.n, self.m, 0);
        for j in range(0, n) {
            let i = r + 1;

            if self.column_iter(j).skip(i).all(|e| *e == Zero::zero()) {
                debug!("reduce: matrix is zeros in col {} from row {}", j, i);
                continue
            }

            if i < m+1 {
                r += 1;
                self.swap_rows(i, r);
                let scale_factor = one::<T>() / *self.get(r, j);
                self.scale_row(r, scale_factor);

                for k in range(1, m) {
                    debug!("m={}", m);
                    if (k == r) { break; }
                    let cur_item = self.get(r, j);
                    let to_zero = self.get(r, i);
                    debug!("cur_item={:?}, to_zero={:?}", cur_item, to_zero);
                }
            }
        }
    }
}

impl<T: Zero + One + Ord + Eq> Mat2<T> {
    /// Test if this matrix is in Reduced Row-Echelon Form. Mostly useful in the shell or as a
    /// helper for `Mat2::rref` (the conversion function).
    pub fn is_rref(&self) -> bool {
        debug!("is_rref: entry");
        // 1.  If there is a row where every entry is zero, then this row lies below any other row
        //     that contains a nonzero entry.
        let zeroes_not_at_end = self.row_iter().fold((false, false), |tup, row| {
            // tuple is (seen_all_zero, seen_non_zero_after_all_zero)
            if row.iter().all(|x| *x == Zero::zero()) {
                (true, tup.n1())
            } else {
                (tup.n0(), if tup.n0() { true } else { false })
            }
        }).n1();
        if zeroes_not_at_end { debug!("all-zero rows not at end"); return false; }

        let mut last_colidx = 0;
        'row: for (rowidx, row) in self.row_iter().enumerate() {
            let mut seen_leading_one = false;
            for (leftmostidx, val) in row.iter().enumerate() {
                if seen_leading_one { continue 'row; }
                if *val != Zero::zero() {
                    if *val != One::one() {
                        debug!("is_rref: false because first non-zero item isn't one \
                                rowidx={}, row={:?}, val={:?}", rowidx, row, *val);
                        return false;
                    }

                    seen_leading_one = true;
                    if leftmostidx < last_colidx { return false; }
                    last_colidx = leftmostidx;
                    for (colidx, colval) in self.column_iter(leftmostidx).enumerate() {
                        debug!("cols: {:?}", self.column_iter(leftmostidx).to_owned_vec());
                        if colidx != rowidx && *colval != Zero::zero() {
                            // 3.  The leftmost nonzero entry of a row is the only nonzero entry
                            //     in its column.
                            debug!("is_rref: false b/c cond 3 violated with colidx={}, \
                                colval={:?}, leftmostidx={}", colidx, *colval, leftmostidx);
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Mat2;

    #[test]
    fn test_cons() {
        let x: Mat2<int> = Mat2::new(3, 2);

        let y: Option<Mat2<int>> = Mat2::from_vec(~[]);
        assert_eq!(y, None);

        let z: Mat2<int> = Mat2::new_with(3, 2, |_,_| 0);
        assert_eq!(x, z);

        let a: Option<Mat2<int>> = Mat2::from_vec(~[~[1, 2, 3], ~[1, 2]]);
        assert_eq!(a, None);
    }

    #[test]
    fn test_get_dimension() {
        let x: Mat2<int> = Mat2::from_vec(~[~[1, 2], ~[3, 4]]).unwrap();
        assert_eq!(x.get_dimension(), (2, 2));
    }

    #[test]
    fn test_swap_rows() {
        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        x.swap_rows(0, 1);
        assert!(x.get_row(0) == &[4, 5, 6]);
        assert!(x.get_row(1) == &[1, 2, 3]);
        assert!(x.get_row(2) == &[7, 8, 9]);
    }

    #[test]
    fn test_get_row() {
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.get_row_opt(3) == None);
    }

    #[test]
    fn test_append_column() {
        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.append_column(~[0, 0, 0]));
        assert!(x.get_row(0) == &[1, 2, 3, 0]);

        // non-square
        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3, 0],
                ~[4, 5, 6, 0],
                ~[7, 8, 9, 0],
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3, 0]);
        assert!(x.append_column(~[0, 0, 0]));
        assert!(!x.append_column(~[0]));
        assert!(x.get_row(0) == &[1, 2, 3, 0, 0]);

        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9],
                ~[10, 11, 12],
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.append_column(~[0, 0, 0, 0]));
        assert!(!x.append_column(~[0]));
        assert!(x.get_row(0) == &[1, 2, 3, 0]);
    }

    #[test]
    fn test_append_row() {
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.get_row_opt(3) == None);
        let mut x = x;
        assert!(x.append_row(~[10, 11, 12]));
        assert!(!x.append_row(~[0]));
        let x = x;
        assert!(x.get_row(3) == &[10, 11, 12]);

        // non-square
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9],
                ~[10, 11, 12],
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.get_row_opt(4) == None);
        let mut x = x;
        assert!(x.append_row(~[10, 11, 12]));
        assert!(!x.append_row(~[0]));
        let x = x;
        assert!(x.get_row(4) == &[10, 11, 12]);

        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3, 0],
                ~[4, 5, 6, 0],
                ~[7, 8, 9, 0]
            ]).unwrap();
        assert!(x.get_row(0) == &[1, 2, 3, 0]);
        assert!(x.get_row_opt(3) == None);
        let mut x = x;
        assert!(x.append_row(~[10, 11, 12, 13]));
        assert!(!x.append_row(~[0]));
        let x = x;
        assert!(x.get_row(3) == &[10, 11, 12, 13]);
    }

    #[test]
    fn test_row_iter() {
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        let mut it = x.row_iter();
        assert_eq!(it.next().unwrap(), &[1,2,3]);
        assert_eq!(it.next().unwrap(), &[4,5,6]);
        assert_eq!(it.next().unwrap(), &[7,8,9]);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_column_iter() {
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        let mut it = x.column_iter(0);
        assert_eq!(it.next().unwrap(), &1);
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.next().unwrap(), &7);
        assert_eq!(it.next(), None);
        let mut it = x.column_iter(1);
        assert_eq!(it.next().unwrap(), &2);
        assert_eq!(it.next().unwrap(), &5);
        assert_eq!(it.next().unwrap(), &8);
        assert_eq!(it.next(), None);
        let mut it = x.column_iter(2);
        assert_eq!(it.next().unwrap(), &3);
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.next().unwrap(), &9);
        assert_eq!(it.next(), None);
        let mut it = x.column_iter(3);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_augment() {
        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        let y = Mat2::from_vec(
            ~[
                ~[4, 5, 6, 7],
                ~[7, 8, 9, 10],
                ~[10, 11, 12, 13]
            ]).unwrap();
        let z = Mat2::from_vec(
            ~[
                ~[1, 2]
            ]).unwrap();

        assert!(x.augment(y));
        assert!(!x.augment(z));

        let mut it = x.row_iter();
        assert_eq!(it.next().unwrap(), &[1,2,3,4,5,6,7]);
        assert_eq!(it.next().unwrap(), &[4,5,6,7,8,9,10]);
        assert_eq!(it.next().unwrap(), &[7,8,9,10,11,12,13]);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_scale_row() {
        let mut x = Mat2::from_vec(~[~[1i, 1, 1]]).unwrap();
        x.scale_row(0, 3);
        assert!(x.get_row(0) == &[3, 3, 3]);
    }

    #[test]
    fn test_add_scaled() {
        let mut x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[4, 5, 6],
                ~[7, 8, 9]
            ]).unwrap();
        x.add_scaled(0, 1, 1);
        assert!(x.get_row(0) == &[1, 2, 3]);
        assert!(x.get_row(1) == &[5, 7, 9]);
    }

    #[test]
    fn test_is_rref() {
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[1, 5, 6],
                ~[1, 8, 9]
            ]).unwrap();
        assert!(!x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[0, 0, 0],
                ~[1, 8, 9]
            ]).unwrap();
        assert!(!x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[1, 5, 6],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(!x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 2, 3],
                ~[0, 0, 0],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[0i, 0, 0],
                ~[0, 0, 0],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 0, 0],
                ~[0, 1, 0],
                ~[0, 0, 1]
            ]).unwrap();
        assert!(x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 0, 0],
                ~[0, 0, 1],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 1, 0],
                ~[0, 0, 1],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 1, 2],
                ~[0, 0, 1],
                ~[0, 0, 0]
            ]).unwrap();
        assert!(!x.is_rref());
        let x = Mat2::from_vec(
            ~[
                ~[1i, 0, 2],
                ~[0, 1, 6],
                ~[0, 0, 0],
            ]).unwrap();
        assert!(x.is_rref());
    }

    #[test]
    fn test_reduce() {
        let mut x = Mat2::from_vec(
            ~[
                ~[1f64, 2.0, 3.0],
                ~[1.0, 5.0, 6.0],
                ~[1.0, 8.0, 9.0]
            ]).unwrap();
        println!("{}", x);
        x.reduce();
        let mut x = Mat2::from_vec(
            ~[
                ~[1f64, 0.0, 3.0],
                ~[1.0, 0.0, 6.0],
                ~[1.0, 0.0, 9.0]
            ]).unwrap();
        println!("{}", x);
        x.reduce();
    }
}
