//! # Local State and Direction Definitions
//!
//! 定义 8-邻域 Moore 网格方向与机器人局部感知状态 `LocalState`。
//! 遵循 arXiv/Springer 2019 论文规范：
//! - $l_1$: North (0, 1)
//! - $l_2$: North-East (1, 1)
//! - $l_3$: East (1, 0)
//! - $l_4$: South-East (1, -1)
//! - $l_5$: South (0, -1)
//! - $l_6$: South-West (-1, -1)
//! - $l_7$: West (-1, 0)
//! - $l_8$: North-West (-1, 1)

use std::fmt;

/// 8 个离散移动与感知方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl Direction {
    /// 全体 8 个方向列表
    pub const ALL: [Direction; 8] = [
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::SouthEast,
        Direction::South,
        Direction::SouthWest,
        Direction::West,
        Direction::NorthWest,
    ];

    /// 方向对应的网格坐标偏移 $(\Delta x, \Delta y)$
    #[inline(always)]
    pub fn offset(self) -> (i32, i32) {
        match self {
            Direction::North => (0, 1),
            Direction::NorthEast => (1, 1),
            Direction::East => (1, 0),
            Direction::SouthEast => (1, -1),
            Direction::South => (0, -1),
            Direction::SouthWest => (-1, -1),
            Direction::West => (-1, 0),
            Direction::NorthWest => (-1, 1),
        }
    }

    /// 获取反方向（如 North <-> South, NE <-> SW）
    #[inline(always)]
    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }

    /// 从相对坐标偏移 $(\Delta x, \Delta y)$ 解析方向
    pub fn from_offset(dx: i32, dy: i32) -> Option<Direction> {
        match (dx, dy) {
            (0, 1) => Some(Direction::North),
            (1, 1) => Some(Direction::NorthEast),
            (1, 0) => Some(Direction::East),
            (1, -1) => Some(Direction::SouthEast),
            (0, -1) => Some(Direction::South),
            (-1, -1) => Some(Direction::SouthWest),
            (-1, 0) => Some(Direction::West),
            (-1, 1) => Some(Direction::NorthWest),
            _ => None,
        }
    }
}

/// 机器人的局部感知状态（256 种可能的状态，由 8 位二进制掩码表示）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalState(pub u8);

impl LocalState {
    /// 空状态（无任何邻居）
    pub const EMPTY: LocalState = LocalState(0);

    /// 全包围状态（8 个邻居全满）
    pub const SURROUNDED: LocalState = LocalState(0xFF);

    /// 由 8 个布尔值构造状态 $[l_1, \dots, l_8]$
    pub fn from_bits(bits: [bool; 8]) -> Self {
        let mut val = 0u8;
        for (i, &b) in bits.iter().enumerate() {
            if b {
                val |= 1 << i;
            }
        }
        LocalState(val)
    }

    /// 检查指定方向是否存在邻居
    #[inline(always)]
    pub fn has_neighbor(self, dir: Direction) -> bool {
        (self.0 & (1 << (dir as u8))) != 0
    }

    /// 设置指定方向是否有邻居
    pub fn set_neighbor(&mut self, dir: Direction, present: bool) {
        if present {
            self.0 |= 1 << (dir as u8);
        } else {
            self.0 &= !(1 << (dir as u8));
        }
    }

    /// 邻居总数
    #[inline(always)]
    pub fn neighbor_count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// 获取所有邻居的相对坐标列表
    pub fn neighbor_coords(self) -> Vec<(i32, i32)> {
        let mut coords = Vec::with_capacity(8);
        for dir in Direction::ALL {
            if self.has_neighbor(dir) {
                coords.push(dir.offset());
            }
        }
        coords
    }

    /// 将所有邻居划分为连通团簇（Cliques）
    /// 两个邻居在网格上若 Chebyshev 距离 $\le 1$ 则被视为连通
    pub fn count_cliques(self) -> usize {
        let coords = self.neighbor_coords();
        let n = coords.len();
        if n == 0 {
            return 0;
        }

        let mut visited = vec![false; n];
        let mut cliques = 0;

        for i in 0..n {
            if !visited[i] {
                cliques += 1;
                let mut queue = vec![i];
                visited[i] = true;

                while let Some(curr) = queue.pop() {
                    let (cx, cy) = coords[curr];
                    for j in 0..n {
                        if !visited[j] {
                            let (nx, ny) = coords[j];
                            if (cx - nx).abs() <= 1 && (cy - ny).abs() <= 1 {
                                visited[j] = true;
                                queue.push(j);
                            }
                        }
                    }
                }
            }
        }
        cliques
    }

    /// 判定是否为单纯形状态（Simplicial State, Definition 5）：
    /// 邻居构成恰好一个连通团簇（Clique）
    #[inline(always)]
    pub fn is_simplicial(self) -> bool {
        let c = self.neighbor_count();
        c > 0 && self.count_cliques() == 1
    }
}

impl fmt::Binary for LocalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08b}", self.0)
    }
}
