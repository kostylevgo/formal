use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Edge<T> {
    pub to: usize,
    pub value: T,
}

#[derive(Clone, Debug)]
pub struct Graph<T> {
    edges: Vec<Vec<Edge<T>>>
}

impl<T> Graph<T> {
    pub fn new() -> Graph<T> {
        Self {
            edges: Vec::new()
        }
    }

    pub fn with_size(sz: usize) -> Graph<T> {
        Self {
            edges: (0..sz).map(|_| Vec::new()).collect()
        }
    }

    pub fn size(&self) -> usize {
        self.edges.len()
    }

    pub fn len(&self) -> usize {
        self.size()
    }

    pub fn get_edges(&self, vertex: usize) -> &Vec<Edge<T>> {
        &self.edges[vertex]
    }

    pub fn get_edges_mut(&mut self, vertex: usize) -> &mut Vec<Edge<T>> {
        &mut self.edges[vertex]
    }

    pub fn add_vertex(&mut self) -> usize {
        self.edges.push(Vec::new());
        self.size() - 1
    }

    pub fn add_edge(&mut self, from: usize, to: usize, value: T) {
        if to >= self.size() {
            panic!("index out of range");
        }
        self.edges[from].push(Edge {to, value});
    }

    pub fn get_edges_list(&self) -> &Vec<Vec<Edge<T>>> {
        &self.edges
    }

    pub fn get_edges_list_mut(&mut self) -> &mut Vec<Vec<Edge<T>>> {
        &mut self.edges
    }
}

impl<T: PartialEq> Graph<T> {
    pub fn remove_edge(&mut self, from: usize, to: usize, value: &T) {
        let found = self.edges[from].iter().position(|x| x.to == to && x.value == *value);
        match found {
            Some(index) => {self.edges[from].remove(index);},
            None => ()
        }
    }
}

impl<T: Display> Display for Graph<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let size = self.size();
        match write!(f, "size: {}\n", size) {
            Err(some) => return Result::Err(some),
            _ => (),
        }
        for state in 0..size {
            for transition in self.get_edges(state).iter() {
                match write!(f, "<{}, {}> -> {}\n", state, transition.value, transition.to) {
                    Err(some) => return Result::Err(some),
                    _ => (),
                }
            }
        }
        Result::Ok(())
    }
}

pub trait DerivedFromGraph<T> {
    fn get_graph(&self) -> &Graph<T>;

    fn size(&self) -> usize {
        self.get_graph().size()
    }

    fn len(&self) -> usize {
        self.get_graph().len()
    }

    fn get_edges(&self, vertex: usize) -> &Vec<Edge<T>> {
        self.get_graph().get_edges(vertex)
    }

    fn get_edges_list(&self) -> &Vec<Vec<Edge<T>>> {
        self.get_graph().get_edges_list()
    }
}

pub trait DerivedFromGraphMut<T: PartialEq>: DerivedFromGraph<T> {
    fn get_graph_mut(&mut self) -> &mut Graph<T>;

    fn add_vertex(&mut self) -> usize {
        self.get_graph_mut().add_vertex()
    }

    fn add_edge(&mut self, from: usize, to: usize, value: T) {
        self.get_graph_mut().add_edge(from, to, value);
    }

    fn remove_edge(&mut self, from: usize, to: usize, value: &T) {
        self.get_graph_mut().remove_edge(from, to, value);
    }

    fn get_edges_mut(&mut self, vertex: usize) -> &mut Vec<Edge<T>> {
        self.get_graph_mut().get_edges_mut(vertex)
    }

    fn get_edges_list_mut(&mut self) -> &mut Vec<Vec<Edge<T>>> {
        self.get_graph_mut().get_edges_list_mut()
    }
}

impl<T> DerivedFromGraph<T> for Graph<T> {
    fn get_graph(&self) -> &Graph<T> {
        self
    }
}

impl<T: PartialEq> DerivedFromGraphMut<T> for Graph<T> {
    fn get_graph_mut(&mut self) -> &mut Graph<T> {
        self
    }
}

impl<T> Graph<T> {
    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut((usize, usize, &T)) -> bool
    {
        for i in 0..self.size() {
            self.edges[i].retain(|x| pred((i, x.to, &x.value)));
        }
    }
}

impl<T: Clone> Graph<T> {
    pub fn reversed_graph(self) -> Self {
        let mut res = Self::with_size(self.size());
        for (i, edges) in self.edges.into_iter().enumerate() {
            for edge in edges.into_iter() {
                res.add_edge(edge.to, i, edge.value);
            }
        }
        res
    }

    pub fn kosaraju(&self) -> Vec<usize> {
        // Finds strongly connected components
        // Returns color: color[u] == color[v] <=> u and v are in the same component
        let my_clone = (*self).clone();
        let reverse_graph = my_clone.reversed_graph();
        let mut used = vec![false; self.size()];
        let mut pseudo_top_sort = Vec::<usize>::new();
        fn pseudo_top_sort_dfs<U>(graph: &Graph<U>, used: &mut Vec<bool>,
                result: &mut Vec<usize>, state: usize) {
            used[state] = true;
            for next_state in graph.edges[state].iter() {
                if !used[next_state.to] {
                    pseudo_top_sort_dfs(graph, used, result, next_state.to);
                }
            }
            result.push(state);
        }
        for state in 0..self.size() {
            if !used[state] {
                pseudo_top_sort_dfs(&reverse_graph, &mut used,
                    &mut pseudo_top_sort, state);
            }
        }
        let mut color = vec![usize::MAX; self.size()];
        let mut color_counter = 0;
        fn coloring_dfs<U>(graph: &Graph<U>, colors: &mut Vec<usize>, state: usize, color: usize) {
            colors[state] = color;
            for next_state in graph.edges[state].iter() {
                if colors[next_state.to] == usize::MAX {
                    coloring_dfs(graph, colors, next_state.to, color);
                }
            }
        }
        for state in pseudo_top_sort.iter().rev() {
            if color[*state] == usize::MAX {
                coloring_dfs(self, &mut color, *state, color_counter);
                color_counter += 1;
            }
        }
        color
    }
}

impl<T: Ord> Graph<T> {
    pub fn remove_equal_edges(&mut self, vertex: usize) {
        self.edges[vertex].sort();
        self.edges[vertex].dedup();
    }

    pub fn compress(self, color: &Vec<usize>) -> Self {
        let mx = color.iter().fold(color[0], |a, b| std::cmp::max(a, *b));
        let mut res = Self::with_size(mx + 1);
        for (i, edges) in self.edges.into_iter().enumerate() {
            for edge in edges.into_iter() {
                res.add_edge(color[i], color[edge.to], edge.value);
            }
        }
        for vertex in 0..res.size() {
            res.remove_equal_edges(vertex);
        }
        res
    }
}