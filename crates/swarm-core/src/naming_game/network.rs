//! # 命名博弈复杂网络拓扑模块 (Network Topologies for Naming Game)
//!
//! 支持以下三类核心拓扑：
//! 1. `CompleteGraph`: 完全图（全连接平均场 Mean-Field 拓扑）
//! 2. `ErdosRenyiGraph`: 同质随机图（Erdös-Rényi，节点度围绕均值呈泊松分布）
//! 3. `BarabasiAlbertGraph`: 异质无标度网络（Barabási-Albert 优先连接模型，具有 Hub 节点）

use rand::Rng;

/// 复杂网络统一特征接口
pub trait Network: Send + Sync {
    /// 节点总数
    fn num_nodes(&self) -> usize;

    /// 节点 i 的度（邻居数量）
    fn degree(&self, node: usize) -> usize;

    /// 获取节点 i 的所有邻居列表
    fn neighbors(&self, node: usize) -> &[usize];

    /// 随机选择节点 i 的一个邻居（均匀随机）
    fn random_neighbor<R: Rng>(&self, node: usize, rng: &mut R) -> usize {
        let nbs = self.neighbors(node);
        assert!(!nbs.is_empty(), "节点 {} 没有邻居", node);
        let idx = rng.gen_range(0..nbs.len());
        nbs[idx]
    }
}

// =========================================================================
// 1. 完全图 (Complete Graph KN)
// =========================================================================

/// 全连接完全图：任意两个不同节点之间均存在无向边
#[derive(Clone, Debug)]
pub struct CompleteGraph {
    pub n: usize,
    // 为统一 neighbors 切片借用接口，每个节点缓存所有其它节点的列表 [0..n \ {i}]
    adj_lists: Vec<Vec<usize>>,
}

impl CompleteGraph {
    pub fn new(n: usize) -> Self {
        assert!(n >= 2, "完全图至少需要 2 个节点");
        let mut adj_lists = Vec::with_capacity(n);
        for i in 0..n {
            let mut nbs = Vec::with_capacity(n - 1);
            for j in 0..n {
                if i != j {
                    nbs.push(j);
                }
            }
            adj_lists.push(nbs);
        }
        Self { n, adj_lists }
    }
}

impl Network for CompleteGraph {
    fn num_nodes(&self) -> usize {
        self.n
    }

    fn degree(&self, _node: usize) -> usize {
        self.n - 1
    }

    fn neighbors(&self, node: usize) -> &[usize] {
        &self.adj_lists[node]
    }

    #[inline]
    fn random_neighbor<R: Rng>(&self, node: usize, rng: &mut R) -> usize {
        // O(1) 直接生成随机节点，避免切片查找
        let offset = rng.gen_range(1..self.n);
        (node + offset) % self.n
    }
}

// =========================================================================
// 2. 邻接表通用图表示
// =========================================================================

/// 基于邻接表的通用无向图
#[derive(Clone, Debug)]
pub struct AdjacencyGraph {
    pub n: usize,
    pub adj_lists: Vec<Vec<usize>>,
}

impl Network for AdjacencyGraph {
    fn num_nodes(&self) -> usize {
        self.n
    }

    fn degree(&self, node: usize) -> usize {
        self.adj_lists[node].len()
    }

    fn neighbors(&self, node: usize) -> &[usize] {
        &self.adj_lists[node]
    }
}

impl AdjacencyGraph {
    /// 添加一条无向边 (u, v)
    pub fn add_edge(&mut self, u: usize, v: usize) {
        if u == v {
            return;
        }
        if !self.adj_lists[u].contains(&v) {
            self.adj_lists[u].push(v);
        }
        if !self.adj_lists[v].contains(&u) {
            self.adj_lists[v].push(u);
        }
    }

    /// 构造同质 Erdös-Rényi 随机图 G(N, p)
    /// - `avg_degree`: 目标平均度 <k>，连接概率 p = <k> / (N - 1)
    pub fn erdos_renyi<R: Rng>(n: usize, avg_degree: f64, rng: &mut R) -> Self {
        assert!(n >= 2);
        let p = (avg_degree / (n - 1) as f64).clamp(0.0, 1.0);
        let mut graph = Self {
            n,
            adj_lists: vec![Vec::new(); n],
        };

        for i in 0..n {
            for j in (i + 1)..n {
                if rng.gen_bool(p) {
                    graph.add_edge(i, j);
                }
            }
        }

        // 确保没有孤立节点（每个孤立节点随机连接一个邻居）
        for i in 0..n {
            if graph.adj_lists[i].is_empty() {
                let target = loop {
                    let cand = rng.gen_range(0..n);
                    if cand != i {
                        break cand;
                    }
                };
                graph.add_edge(i, target);
            }
        }

        graph
    }

    /// 构造异质 Barabási-Albert 无标度网络 (BA 优先连接)
    /// - `m0`: 初始全连通集团节点数 (m0 >= m)
    /// - `m`: 每次新加入一个节点时，连向已有节点的边数
    pub fn barabasi_albert<R: Rng>(n: usize, m0: usize, m: usize, rng: &mut R) -> Self {
        assert!(m0 >= m && m >= 1 && n >= m0, "参数需满足 n >= m0 >= m >= 1");
        let mut graph = Self {
            n,
            adj_lists: vec![Vec::new(); n],
        };

        // 维护用于优先连接轮盘赌的重复节点数组 (repeated_nodes)
        let mut repeated_nodes = Vec::new();

        // 1. 初始化包含 m0 个节点的完全子图
        for i in 0..m0 {
            for j in (i + 1)..m0 {
                graph.add_edge(i, j);
                repeated_nodes.push(i);
                repeated_nodes.push(j);
            }
        }

        // 2. 依次添加新节点 i in m0..n
        for i in m0..n {
            let mut targets = std::collections::HashSet::with_capacity(m);
            while targets.len() < m {
                let rand_idx = rng.gen_range(0..repeated_nodes.len());
                let candidate = repeated_nodes[rand_idx];
                targets.insert(candidate);
            }

            for &t in &targets {
                graph.add_edge(i, t);
                repeated_nodes.push(i);
                repeated_nodes.push(t);
            }
        }

        graph
    }

    /// 构造一维规则环形网络 (Regular Ring Lattice)
    /// - 每个节点连向左右各 k/2 个邻居（总度数为 k，k 为偶数）
    pub fn regular_ring(n: usize, k: usize) -> Self {
        assert!(n > k && k >= 2 && k % 2 == 0, "n > k 且 k 必须为正偶数");
        let mut graph = Self {
            n,
            adj_lists: vec![Vec::new(); n],
        };
        let half_k = k / 2;
        for i in 0..n {
            for d in 1..=half_k {
                let j = (i + d) % n;
                graph.add_edge(i, j);
            }
        }
        graph
    }

    /// 构造小世界网络 (Watts-Strogatz Small-World Model)
    /// - `k`: 初始规则环邻居数（偶数）
    /// - `p`: 边重连概率 p in [0, 1]
    pub fn watts_strogatz<R: Rng>(n: usize, k: usize, p: f64, rng: &mut R) -> Self {
        assert!(n > k && k >= 2 && k % 2 == 0);
        let mut graph = Self::regular_ring(n, k);
        if p <= 0.0 {
            return graph;
        }

        let half_k = k / 2;
        for i in 0..n {
            for d in 1..=half_k {
                let original_target = (i + d) % n;
                if rng.gen_bool(p.clamp(0.0, 1.0)) {
                    // 尝试重连到随机目标节点
                    let mut attempts = 0;
                    while attempts < 20 {
                        let cand = rng.gen_range(0..n);
                        if cand != i && !graph.adj_lists[i].contains(&cand) {
                            // 移除原边
                            if let Some(pos) = graph.adj_lists[i].iter().position(|&x| x == original_target) {
                                graph.adj_lists[i].swap_remove(pos);
                            }
                            if let Some(pos) = graph.adj_lists[original_target].iter().position(|&x| x == i) {
                                graph.adj_lists[original_target].swap_remove(pos);
                            }
                            // 添加新边
                            graph.add_edge(i, cand);
                            break;
                        }
                        attempts += 1;
                    }
                }
            }
        }
        graph
    }
}

