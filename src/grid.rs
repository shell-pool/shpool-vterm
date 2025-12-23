// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The grid module defines a view of the terminal where every cell is
//! assigned to a definite row and column. It is used at dump time to
//! dump the precise information requested and is also used to display
//! some commands.

use crate::cell::Cell;

// A grid line is a list of slices into the underlying scrollback.
// A sorted list of starting indexes is maintained to support O(lg(n))
// indexing.
//
// The CellSlice type param is either a `&'scrollback [Cell]` or a
// `&'scrollback mut [Cell]`. We just use this pattern to be able
// to add mutable methods 
struct Line<'scrollback> {
    /// The column indexes at which each cell slice starts. This list is kept
    /// sorted, and is used to binary search for the right slice while indexing.
    start_offsets: Vec<usize>,
    cells: Vec<Vec<&'scrollback Cell>>,
    capacity: usize,
    empty: &'scrollback Cell,
}

impl<'scrollback> Line<'scrollback>
{
    /// An empty line of the given length.
    pub fn new(empty: &'scrollback Cell, capacity: usize) -> Self {
        Line {
            start_offsets: vec![],
            cells: vec![],
            capacity,
            empty: empty,
        }
    }

    // Add a new chunk of cells to the grid line.
    // 
    // Returns: the number of cells from the chunk added to the line.
    pub fn add_chunk(&mut self, chunk: &'scrollback [Cell]) -> usize
    {
        assert_eq!(self.cells.len(), self.start_offsets.len(), "out of sync vecs");

        let mut consumed = 0;
        let next_chunk = {
            let current_len = self.len();
            let remaining_slots = self.capacity - current_len;
            let mut total_width = 0;
            while total_width < remaining_slots && consumed < chunk.len() {
                total_width += chunk[consumed].width() as usize;
                consumed += 1;
            }

            // Pull back from any wide cells at the end.
            let mut fill_to_end_with_empties = false;
            if current_len + total_width > self.capacity {
                while current_len + total_width > self.capacity {
                    total_width -= chunk[consumed].width() as usize;
                    consumed -= 1;
                }
                fill_to_end_with_empties = true;
            }

            // wide chars take up multiple grid slots, so we store a reference to the
            // cell multiple times to ensure indexing works as expected.
            chunk[..consumed].iter()
                .flat_map(|cell| std::iter::repeat(cell).take(cell.width() as usize))
                .chain(
                    if fill_to_end_with_empties {
                        Some(std::iter::repeat(self.empty).take(self.capacity - (current_len + total_width)))
                    } else {
                        None
                    }.into_iter().flatten()
                )
                .collect::<Vec<&Cell>>()
        };

        self.start_offsets.push(self.len());
        self.cells.push(next_chunk);

        consumed
    }

    pub fn len(&self) -> usize {
        if self.cells.is_empty() {
            0
        } else {
            // N.B. though some cells can be variable width, in a grid view,
            // variable width cells take up multiple grid slots, so we can just
            // directly use slice length here to figure out the next starting
            // offset.
            self.start_offsets[self.start_offsets.len()-1] +
                self.cells[self.cells.len()-1].len()
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn get(&self, index: usize) -> Option<&Cell> {
        if index >= self.len() {
            return None;
        }

        let mut low = 0;
        let mut high = self.start_offsets.len();
        while low + 1 != high {
            let probe = low + ((high - low) / 2);

            if index >= self.start_offsets[probe] {
                low = probe;
                if probe == self.start_offsets.len() - 1 || self.start_offsets[probe + 1] > index {
                    break;
                }
            } else {
                high = probe;
            }
        }

        Some(&self.cells[low][index - self.start_offsets[low]])
    }
}

impl<'scrollback> std::ops::Index<usize> for Line<'scrollback> {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        if let Some(v) = self.get(index) {
            return v;
        }
        panic!("index {} out of bounds (len={})", index, self.len());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cell::Cell;

    fn make_cells(chars: &str) -> Vec<Cell> {
        chars.chars().map(|c| Cell::new(c)).collect()
    }

    #[test]
    fn test_sanity_accessors() {
        let empty_cell = Cell::empty();
        let line = Line::new(&empty_cell, 10);
        assert_eq!(line.len(), 0);
        assert_eq!(line.capacity(), 10);
        assert!(line.get(0).is_none());
    }

    #[test]
    fn test_single_chunk() {
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 10);
        let cells = make_cells("abc");
        let consumed = line.add_chunk(&cells);

        assert_eq!(consumed, 3, "should consume 3 cells");
        assert_eq!(line.len(), 3, "line length should be 3");

        let c0 = &line[0];
        assert!(format!("{:?}", c0).contains("'a'"));

        let c1 = &line[1];
        assert!(format!("{:?}", c1).contains("'b'"));

        let c2 = &line[2];
        assert!(format!("{:?}", c2).contains("'c'"));
    }

    #[test]
    fn test_multiple_chunks() {
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 10);
        let c1 = make_cells("ab");
        let c2 = make_cells("cd");

        line.add_chunk(&c1);
        line.add_chunk(&c2);

        assert_eq!(line.len(), 4);

        assert!(format!("{:?}", &line[0]).contains("'a'"));
        assert!(format!("{:?}", &line[1]).contains("'b'"));
        assert!(format!("{:?}", &line[2]).contains("'c'"));
        assert!(format!("{:?}", &line[3]).contains("'d'"));
    }

    #[test]
    fn test_capacity_limit_big_chunk() {
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 3);
        let cells = make_cells("abcde");
        let consumed = line.add_chunk(&cells);

        assert_eq!(consumed, 3, "should consume only 3 cells");
        assert_eq!(line.len(), 3, "len should be capped at 3");

        assert!(format!("{:?}", &line[2]).contains("'c'"));
        assert!(line.get(3).is_none());
    }

    #[test]
    fn test_capacity_limit_small_chunks() {
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 3);
        let c1 = make_cells("a");
        line.add_chunk(&c1);
        let c2 = make_cells("b");
        line.add_chunk(&c2);
        let c3 = make_cells("c");
        line.add_chunk(&c3);
        let c4 = make_cells("d");
        let consumed = line.add_chunk(&c4);

        assert_eq!(consumed, 0, "should consume 0 cells when full");
        assert_eq!(line.len(), 3);
        assert!(format!("{:?}", &line[2]).contains("'c'"));
    }

    #[test]
    fn test_out_of_bounds() {
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 5);
        let c1 = make_cells("a");
        line.add_chunk(&c1);
        assert!(line.get(1).is_none());
        assert!(line.get(5).is_none());
    }

    #[test]
    fn test_wide_chars() {
        // '螃' is wide.
        let empty_cell = Cell::empty();
        let mut line = Line::new(&empty_cell, 4);
        let cells = make_cells("a螃b"); // '螃' usually width 2
        // If width is 2:
        // 'a' -> index 0
        // '螃' -> index 1, 2
        // 'b' -> index 3

        let consumed = line.add_chunk(&cells);
        assert_eq!(consumed, 3);
        assert_eq!(line.len(), 4);

        assert!(format!("{:?}", &line[0]).contains("'a'"));

        let c1 = &line[1];
        // c1 should be the wide char
        assert_eq!(c1.width(), 2);

        let c2 = &line[2];
        // c2 should be the same reference/content as c1
        assert_eq!(c2.width(), 2);

        assert!(format!("{:?}", &line[3]).contains("'b'"));
    }
}
