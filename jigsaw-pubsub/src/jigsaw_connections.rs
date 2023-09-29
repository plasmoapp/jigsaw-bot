use std::cmp::Ordering;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use tiny_skia::{FillRule, Mask, PathBuilder, Transform};
use crate::jigsaw_connections::PieceConnection::*;

#[derive(Clone, Debug)]
pub struct PieceConnections {
    pub top: PieceConnection,
    pub right: PieceConnection,
    pub bottom: PieceConnection,
    pub left: PieceConnection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PieceConnection {
    ConnectedToNeighbor,
    NeighborConnected,
    Nothing
}

impl PieceConnections {
    fn append_horizontal_connection_to_mask(
        pb: &mut PathBuilder,
        tile_size: f32,
        connection_percent: f32,
        offset_x: f32,
        offset_y: f32,
        flip: bool,
        reverse: bool
    ) {
        let flip = match reverse { true => !flip, false => flip };

        let multiply_tile_size_x = |multiplier: f32| -> f32 {
            offset_x + (tile_size * multiplier)
        };

        let multiply_tile_size_y = |multiplier: f32, flip: bool| -> f32 {
            offset_y + (tile_size * multiplier * match flip { true => -1.0, false => 1.0 })
        };

        if reverse {
            pb.cubic_to(
                multiply_tile_size_x(0.8),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5 + connection_percent),
                multiply_tile_size_y(connection_percent, flip)
            );
            pb.cubic_to(
                multiply_tile_size_x(0.5 + 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 - 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 - connection_percent),
                multiply_tile_size_y(connection_percent, flip)
            );
            pb.cubic_to(
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.2),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.0),
                multiply_tile_size_y(0.0, flip)
            );
        } else {
            pb.cubic_to(
                multiply_tile_size_x(0.2),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5 - connection_percent),
                multiply_tile_size_y(connection_percent, flip)
            );
            pb.cubic_to(
                multiply_tile_size_x(0.5 - 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 + 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 + connection_percent),
                multiply_tile_size_y(connection_percent, flip)
            );
            pb.cubic_to(
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.8),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(1.0),
                multiply_tile_size_y(0.0, flip)
            );
        }
    }

    fn append_vertical_connection_to_mask(
        pb: &mut PathBuilder,
        tile_size: f32,
        connection_percent: f32,
        offset_x: f32,
        offset_y: f32,
        flip: bool,
        reverse: bool
    ) {
        let flip = match reverse { true => !flip, false => flip };

        let multiply_tile_size_x = |multiplier: f32| -> f32 {
            offset_x + (tile_size * multiplier)
        };

        let multiply_tile_size_y = |multiplier: f32, flip: bool| -> f32 {
            offset_y + (tile_size * multiplier * match flip { true => -1.0, false => 1.0 })
        };

        if reverse {
            pb.cubic_to(
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.8),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(connection_percent, flip),
                multiply_tile_size_x(0.5 + connection_percent)
            );
            pb.cubic_to(
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 + 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 - 2.0 * connection_percent),
                multiply_tile_size_y(connection_percent, flip),
                multiply_tile_size_x(0.5 - connection_percent)
            );
            pb.cubic_to(
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.2),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.0)
            );
        } else {
            pb.cubic_to(
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.2),
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(connection_percent, flip),
                multiply_tile_size_x(0.5 - connection_percent)
            );
            pb.cubic_to(
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 - 2.0 * connection_percent),
                multiply_tile_size_y(3.0 * connection_percent, flip),
                multiply_tile_size_x(0.5 + 2.0 * connection_percent),
                multiply_tile_size_y(connection_percent, flip),
                multiply_tile_size_x(0.5 + connection_percent)
            );
            pb.cubic_to(
                multiply_tile_size_y(-connection_percent, flip),
                multiply_tile_size_x(0.5),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(0.8),
                multiply_tile_size_y(0.0, flip),
                multiply_tile_size_x(1.0)
            );
        }
    }

    pub fn create_piece_mask(&self, width: u32, height: u32, tile_size: u32) -> Mask {
        let clip_path = {
            let mut pb = PathBuilder::new();

            let tile_size = tile_size as f32;

            let offset = ((width as f32) - tile_size) / 2.0;

            let connection_percent = 0.125;

            // top left corner
            pb.move_to(offset, offset);

            // top side
            if self.top != Nothing {
                Self::append_horizontal_connection_to_mask(
                    &mut pb,
                    tile_size,
                    connection_percent,
                    offset,
                    offset,
                    self.top == ConnectedToNeighbor,
                    false
                );
            } else {
                pb.line_to(offset + tile_size, offset);
            }

            // right side
            if self.right != Nothing {
                Self::append_vertical_connection_to_mask(
                    &mut pb,
                    tile_size,
                    connection_percent,
                    offset,
                    offset + tile_size,
                    self.right == ConnectedToNeighbor,
                    false
                );
            } else {
                pb.line_to(offset + tile_size, offset + tile_size);
            }

            // bottom
            if self.bottom != Nothing {
                Self::append_horizontal_connection_to_mask(
                    &mut pb,
                    tile_size,
                    connection_percent,
                    offset,
                    offset + tile_size,
                    self.bottom == ConnectedToNeighbor,
                    true
                );
            } else {
                pb.line_to(offset, offset + tile_size);
            }

            // left side
            if self.left != Nothing {
                Self::append_vertical_connection_to_mask(
                    &mut pb,
                    tile_size,
                    connection_percent,
                    offset,
                    offset,
                    self.left == ConnectedToNeighbor,
                    true
                );
            } else {
                pb.line_to(offset, offset);
            }

            pb.finish().unwrap()
        };

        let mut mask = Mask::new(width, height).unwrap();
        mask.fill_path(&clip_path, FillRule::EvenOdd, true, Transform::default());

        mask
    }
}

impl PieceConnections {
    fn generate_connection(rng: &mut StdRng, should_generate: bool) -> PieceConnection {
        match should_generate {
            true => match rng.gen_bool(0.5) {
                true => ConnectedToNeighbor,
                false => NeighborConnected
            }
            false => Nothing
        }
    }

    /// Generates a matrix of PieceConnection with specified width, height and seed.
    pub fn generate_connections_for_size(width_px: u32, height_px: u32, tile_size_px: u32, seed: Option<u64>) -> Vec<Vec<PieceConnections>>  {
        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy()
        };

        let width = (width_px / tile_size_px) as usize;
        let height = (height_px / tile_size_px) as usize;

        let mut matrix: Vec<Vec<Option<PieceConnections>>> = vec![
            vec![None; width];
            height
        ];

        for y in 0..height {
            for x in 0..width {
                let mut connection = PieceConnections {
                    top: Self::generate_connection(&mut rng, y > 0),
                    right: Self::generate_connection(&mut rng, x < width - 1),
                    bottom: Self::generate_connection(&mut rng, y < height - 1),
                    left: Self::generate_connection(&mut rng, x > 0)
                };

                let top_neighbor = match y.cmp(&0) {
                    Ordering::Greater =>
                        matrix.get(y - 1)
                            .map_or(None, |row| row.get(x))
                            .unwrap_or(&None),
                    _ => &None
                };
                let right_neighbor = match x.cmp(&(width - 1)) {
                    Ordering::Less =>
                        matrix.get(y)
                            .map_or(None, |row| row.get(x + 1))
                            .unwrap_or(&None),
                    _ => &None
                };
                let bottom_neighbor = match y.cmp(&(height - 1)) {
                    Ordering::Less =>
                        matrix.get(y + 1)
                            .map_or(None, |row| row.get(x))
                            .unwrap_or(&None),
                    _ => &None
                };
                let left_neighbor = match x.cmp(&0) {
                    Ordering::Greater =>
                        matrix.get(y)
                            .map_or(None, |row| row.get(x - 1))
                            .unwrap_or(&None),
                    _ => &None
                };


                match top_neighbor {
                    Some(neighbor) => {
                        if neighbor.bottom == ConnectedToNeighbor {
                            connection.top = NeighborConnected;
                        } else {
                            connection.top = ConnectedToNeighbor;
                        }
                    }
                    _ => {}
                }
                match right_neighbor {
                    Some(neighbor) => {
                        if neighbor.left == ConnectedToNeighbor {
                            connection.right = NeighborConnected;
                        } else {
                            connection.right = ConnectedToNeighbor;
                        }
                    }
                    _ => {}
                }
                match bottom_neighbor {
                    Some(neighbor) => {
                        if neighbor.top == ConnectedToNeighbor {
                            connection.bottom = NeighborConnected;
                        } else {
                            connection.bottom = ConnectedToNeighbor;
                        }
                    }
                    _ => {}
                }
                match left_neighbor {
                    Some(neighbor) => {
                        if neighbor.right == ConnectedToNeighbor {
                            connection.left = NeighborConnected;
                        } else {
                            connection.left = ConnectedToNeighbor;
                        }
                    }
                    _ => {}
                }

                matrix[y][x] = Some(connection);
            }
        }

        matrix
            .iter()
            .map(|row|
                     row.iter().map(|element| element.clone().unwrap()).collect()
            )
            .collect()
    }
}
