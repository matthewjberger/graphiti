use crate::schema::Direction;
use nalgebra_glm::{Vec2, vec2};

#[derive(Clone, Debug)]
pub struct GraphInput {
    pub node_sizes: Vec<Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub direction: Direction,
    pub rank_gap: f32,
    pub sibling_gap: f32,
    pub edge_lane: f32,
    pub node_group: Vec<Option<usize>>,
    pub group_padding: f32,
}

#[derive(Clone, Debug, Default)]
pub struct GraphLayout {
    pub positions: Vec<Vec2>,
    pub size: Vec2,
    pub edge_waypoints: Vec<Vec<Vec2>>,
    pub reversed: Vec<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotKind {
    Real(usize),
    Virtual(usize),
}

struct Layered {
    kind: Vec<SlotKind>,
    rank: Vec<usize>,
    cross_size: Vec<f32>,
    rank_size: Vec<f32>,
    order: Vec<Vec<usize>>,
    successors: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
    chains: Vec<Vec<usize>>,
}

pub fn layout_layered(input: &GraphInput) -> GraphLayout {
    let node_count = input.node_sizes.len();
    if node_count == 0 {
        return GraphLayout::default();
    }

    let (acyclic, reversed) = break_cycles(node_count, &input.edges);
    let ranks = assign_ranks(node_count, &acyclic);
    let mut layered = build_layered(input, &acyclic, &ranks);
    order_slots(&mut layered);
    compact_groups(&mut layered, &input.node_group);
    let mut cross = assign_cross_positions(&layered, input);
    separate_groups(&layered, input, &mut cross);
    let (rank_offsets, rank_extents) = assign_rank_positions(&layered, input);

    let mut positions = vec![vec2(0.0, 0.0); node_count];
    let mut slot_center = vec![vec2(0.0, 0.0); layered.kind.len()];
    for slot in 0..layered.kind.len() {
        let rank = layered.rank[slot];
        let rank_center = rank_offsets[rank] + rank_extents[rank] * 0.5;
        slot_center[slot] = vec2(cross[slot], rank_center);
        if let SlotKind::Real(node) = layered.kind[slot] {
            let size = input.node_sizes[node];
            let (cross_size, rank_size) = split_size(size, input.direction);
            let top_left_cross = cross[slot] - cross_size * 0.5;
            let top_left_rank = rank_offsets[rank] + (rank_extents[rank] - rank_size) * 0.5;
            positions[node] = combine(top_left_cross, top_left_rank, input.direction);
        }
    }

    let mut edge_waypoints = Vec::with_capacity(input.edges.len());
    for (index, chain) in layered.chains.iter().enumerate() {
        let mut points: Vec<Vec2> = chain
            .iter()
            .map(|&slot| {
                let center = slot_center[slot];
                combine_point(center.x, center.y, input.direction)
            })
            .collect();
        if reversed[index] {
            points.reverse();
        }
        edge_waypoints.push(points);
    }

    let mut bounds_min = vec2(f32::MAX, f32::MAX);
    let mut bounds_max = vec2(f32::MIN, f32::MIN);
    for (node, position) in positions.iter().enumerate() {
        let size = input.node_sizes[node];
        bounds_min.x = bounds_min.x.min(position.x);
        bounds_min.y = bounds_min.y.min(position.y);
        bounds_max.x = bounds_max.x.max(position.x + size.x);
        bounds_max.y = bounds_max.y.max(position.y + size.y);
    }
    for path in &edge_waypoints {
        for point in path {
            bounds_min.x = bounds_min.x.min(point.x);
            bounds_min.y = bounds_min.y.min(point.y);
            bounds_max.x = bounds_max.x.max(point.x);
            bounds_max.y = bounds_max.y.max(point.y);
        }
    }

    for position in positions.iter_mut() {
        *position -= bounds_min;
    }
    for path in edge_waypoints.iter_mut() {
        for point in path.iter_mut() {
            *point -= bounds_min;
        }
    }

    GraphLayout {
        positions,
        size: bounds_max - bounds_min,
        edge_waypoints,
        reversed,
    }
}

fn break_cycles(node_count: usize, edges: &[(usize, usize)]) -> (Vec<(usize, usize)>, Vec<bool>) {
    let mut adjacency = vec![Vec::new(); node_count];
    for (index, &(from, to)) in edges.iter().enumerate() {
        if from < node_count && to < node_count {
            adjacency[from].push((to, index));
        }
    }
    let mut state = vec![0u8; node_count];
    let mut reversed = vec![false; edges.len()];
    for start in 0..node_count {
        if state[start] == 0 {
            depth_first(start, &adjacency, &mut state, &mut reversed);
        }
    }
    let acyclic = edges
        .iter()
        .enumerate()
        .map(|(index, &(from, to))| {
            if reversed[index] {
                (to, from)
            } else {
                (from, to)
            }
        })
        .collect();
    (acyclic, reversed)
}

fn depth_first(
    node: usize,
    adjacency: &[Vec<(usize, usize)>],
    state: &mut Vec<u8>,
    reversed: &mut Vec<bool>,
) {
    state[node] = 1;
    for &(next, edge) in &adjacency[node] {
        match state[next] {
            1 => reversed[edge] = true,
            0 => depth_first(next, adjacency, state, reversed),
            _ => {}
        }
    }
    state[node] = 2;
}

fn assign_ranks(node_count: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut incoming = vec![0usize; node_count];
    let mut adjacency = vec![Vec::new(); node_count];
    for &(from, to) in edges {
        if from == to {
            continue;
        }
        adjacency[from].push(to);
        incoming[to] += 1;
    }
    let mut ranks = vec![0usize; node_count];
    let mut queue: Vec<usize> = (0..node_count)
        .filter(|&node| incoming[node] == 0)
        .collect();
    let mut visited = 0;
    while let Some(node) = queue.pop() {
        visited += 1;
        for &next in &adjacency[node] {
            if ranks[next] < ranks[node] + 1 {
                ranks[next] = ranks[node] + 1;
            }
            incoming[next] -= 1;
            if incoming[next] == 0 {
                queue.push(next);
            }
        }
    }
    if visited < node_count {
        for &(from, to) in edges {
            if from != to && ranks[to] <= ranks[from] {
                ranks[to] = ranks[from] + 1;
            }
        }
    }
    ranks
}

fn build_layered(input: &GraphInput, edges: &[(usize, usize)], ranks: &[usize]) -> Layered {
    let node_count = input.node_sizes.len();
    let rank_count = ranks.iter().copied().max().unwrap_or(0) + 1;
    let mut kind = Vec::with_capacity(node_count);
    let mut slot_rank = Vec::with_capacity(node_count);
    let mut cross_size = Vec::with_capacity(node_count);
    let mut rank_size = Vec::with_capacity(node_count);
    for (node, rank) in ranks.iter().enumerate().take(node_count) {
        let (cross, along) = split_size(input.node_sizes[node], input.direction);
        kind.push(SlotKind::Real(node));
        slot_rank.push(*rank);
        cross_size.push(cross);
        rank_size.push(along);
    }

    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut chains: Vec<Vec<usize>> = Vec::with_capacity(edges.len());

    for (edge_index, &(from, to)) in edges.iter().enumerate() {
        if from == to {
            chains.push(vec![from, to]);
            continue;
        }
        let mut chain = vec![from];
        let from_rank = ranks[from];
        let to_rank = ranks[to];
        let mut previous = from;
        if to_rank > from_rank + 1 {
            for rank in (from_rank + 1)..to_rank {
                let slot = kind.len();
                kind.push(SlotKind::Virtual(edge_index));
                slot_rank.push(rank);
                cross_size.push(input.edge_lane);
                rank_size.push(0.0);
                successors.push(Vec::new());
                predecessors.push(Vec::new());
                successors[previous].push(slot);
                predecessors[slot].push(previous);
                chain.push(slot);
                previous = slot;
            }
        }
        successors[previous].push(to);
        predecessors[to].push(previous);
        chain.push(to);
        chains.push(chain);
    }

    let mut order: Vec<Vec<usize>> = vec![Vec::new(); rank_count];
    for slot in 0..kind.len() {
        order[slot_rank[slot]].push(slot);
    }

    Layered {
        kind,
        rank: slot_rank,
        cross_size,
        rank_size,
        order,
        successors,
        predecessors,
        chains,
    }
}

fn order_slots(layered: &mut Layered) {
    let mut best = layered.order.clone();
    let mut best_crossings = count_crossings(layered, &best);
    for iteration in 0..12 {
        if iteration % 2 == 0 {
            for rank in 1..layered.order.len() {
                sort_by_barycenter(layered, rank, true);
            }
        } else {
            for rank in (0..layered.order.len().saturating_sub(1)).rev() {
                sort_by_barycenter(layered, rank, false);
            }
        }
        transpose(layered);
        let crossings = count_crossings(layered, &layered.order);
        if crossings < best_crossings {
            best_crossings = crossings;
            best = layered.order.clone();
        }
    }
    layered.order = best;
}

fn compact_groups(layered: &mut Layered, node_group: &[Option<usize>]) {
    if node_group.iter().all(|group| group.is_none()) {
        return;
    }
    for rank in 0..layered.order.len() {
        let slots = layered.order[rank].clone();
        let mut sums: std::collections::HashMap<usize, (f32, f32)> =
            std::collections::HashMap::new();
        for (index, &slot) in slots.iter().enumerate() {
            if let SlotKind::Real(node) = layered.kind[slot]
                && let Some(Some(group)) = node_group.get(node)
            {
                let entry = sums.entry(*group).or_insert((0.0, 0.0));
                entry.0 += index as f32;
                entry.1 += 1.0;
            }
        }
        let mut keyed: Vec<(f32, usize, usize)> = slots
            .iter()
            .enumerate()
            .map(|(index, &slot)| {
                let key = match layered.kind[slot] {
                    SlotKind::Real(node) => match node_group.get(node).copied().flatten() {
                        Some(group) => {
                            let (sum, count) = sums[&group];
                            sum / count.max(1.0)
                        }
                        None => index as f32,
                    },
                    SlotKind::Virtual(_) => index as f32,
                };
                (key, index, slot)
            })
            .collect();
        keyed.sort_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(left.1.cmp(&right.1))
        });
        layered.order[rank] = keyed.into_iter().map(|(_, _, slot)| slot).collect();
    }
}

fn sort_by_barycenter(layered: &mut Layered, rank: usize, use_predecessors: bool) {
    let reference: Vec<usize> = if use_predecessors {
        if rank == 0 {
            return;
        }
        layered.order[rank - 1].clone()
    } else {
        if rank + 1 >= layered.order.len() {
            return;
        }
        layered.order[rank + 1].clone()
    };
    let mut index_of = std::collections::HashMap::new();
    for (index, &slot) in reference.iter().enumerate() {
        index_of.insert(slot, index as f32);
    }
    let current = layered.order[rank].clone();
    let mut keyed: Vec<(f32, usize, usize)> = current
        .iter()
        .enumerate()
        .map(|(position, &slot)| {
            let neighbors = if use_predecessors {
                &layered.predecessors[slot]
            } else {
                &layered.successors[slot]
            };
            let values: Vec<f32> = neighbors
                .iter()
                .filter_map(|neighbor| index_of.get(neighbor).copied())
                .collect();
            let key = if values.is_empty() {
                position as f32
            } else {
                values.iter().sum::<f32>() / values.len() as f32
            };
            (key, position, slot)
        })
        .collect();
    keyed.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.1.cmp(&right.1))
    });
    layered.order[rank] = keyed.into_iter().map(|(_, _, slot)| slot).collect();
}

fn transpose(layered: &mut Layered) {
    let mut improved = true;
    let mut guard = 8;
    while improved && guard > 0 {
        guard -= 1;
        improved = false;
        for rank in 0..layered.order.len() {
            let count = layered.order[rank].len();
            for index in 0..count.saturating_sub(1) {
                let before = crossings_around(layered, rank);
                layered.order[rank].swap(index, index + 1);
                let after = crossings_around(layered, rank);
                if after < before {
                    improved = true;
                } else {
                    layered.order[rank].swap(index, index + 1);
                }
            }
        }
    }
}

fn crossings_around(layered: &Layered, rank: usize) -> usize {
    let mut total = 0;
    if rank > 0 {
        total += crossings_between(layered, rank - 1, rank);
    }
    if rank + 1 < layered.order.len() {
        total += crossings_between(layered, rank, rank + 1);
    }
    total
}

fn count_crossings(layered: &Layered, order: &[Vec<usize>]) -> usize {
    let mut total = 0;
    for rank in 0..order.len().saturating_sub(1) {
        total += crossings_between_orders(layered, &order[rank], &order[rank + 1]);
    }
    total
}

fn crossings_between(layered: &Layered, upper: usize, lower: usize) -> usize {
    crossings_between_orders(layered, &layered.order[upper], &layered.order[lower])
}

fn crossings_between_orders(layered: &Layered, upper: &[usize], lower: &[usize]) -> usize {
    let mut position = std::collections::HashMap::new();
    for (index, &slot) in lower.iter().enumerate() {
        position.insert(slot, index);
    }
    let mut targets = Vec::new();
    for &slot in upper {
        let mut local: Vec<usize> = layered.successors[slot]
            .iter()
            .filter_map(|next| position.get(next).copied())
            .collect();
        local.sort_unstable();
        targets.extend(local);
    }
    let mut crossings = 0;
    for left in 0..targets.len() {
        for right in (left + 1)..targets.len() {
            if targets[left] > targets[right] {
                crossings += 1;
            }
        }
    }
    crossings
}

fn assign_cross_positions(layered: &Layered, input: &GraphInput) -> Vec<f32> {
    let slot_count = layered.kind.len();
    let mut cross = vec![0.0f32; slot_count];
    for rank_slots in &layered.order {
        let mut cursor = 0.0;
        for &slot in rank_slots {
            let half = layered.cross_size[slot] * 0.5;
            cursor += half;
            cross[slot] = cursor;
            cursor += half + input.sibling_gap;
        }
    }

    for iteration in 0..24 {
        let downward = iteration % 2 == 0;
        let ranks: Vec<usize> = if downward {
            (0..layered.order.len()).collect()
        } else {
            (0..layered.order.len()).rev().collect()
        };
        for rank in ranks {
            for &slot in &layered.order[rank] {
                let neighbors: Vec<usize> = if downward {
                    layered.predecessors[slot].clone()
                } else {
                    layered.successors[slot].clone()
                };
                if neighbors.is_empty() {
                    continue;
                }
                let mut total = 0.0;
                let mut weight_sum = 0.0;
                for neighbor in neighbors {
                    let weight = match layered.kind[neighbor] {
                        SlotKind::Virtual(_) => 4.0,
                        SlotKind::Real(_) => 1.0,
                    };
                    total += cross[neighbor] * weight;
                    weight_sum += weight;
                }
                let desired = total / weight_sum;
                cross[slot] = cross[slot] + (desired - cross[slot]) * 0.7;
            }
            resolve_overlaps(layered, input, rank, &mut cross);
        }
    }

    let mut minimum = f32::MAX;
    for (slot, value) in cross.iter().enumerate() {
        minimum = minimum.min(value - layered.cross_size[slot] * 0.5);
    }
    for value in cross.iter_mut() {
        *value -= minimum;
    }
    cross
}

fn separate_groups(layered: &Layered, input: &GraphInput, cross: &mut [f32]) {
    let group_count = match input.node_group.iter().filter_map(|group| *group).max() {
        Some(highest) => highest + 1,
        None => return,
    };
    let padding = input.group_padding;
    for group in 0..group_count {
        let members: Vec<usize> = (0..layered.kind.len())
            .filter(|&slot| match layered.kind[slot] {
                SlotKind::Real(node) => {
                    input.node_group.get(node).copied().flatten() == Some(group)
                }
                SlotKind::Virtual(_) => false,
            })
            .collect();
        if members.is_empty() {
            continue;
        }
        let low_rank = members
            .iter()
            .map(|&slot| layered.rank[slot])
            .min()
            .unwrap();
        let high_rank = members
            .iter()
            .map(|&slot| layered.rank[slot])
            .max()
            .unwrap();
        let interval_min = members
            .iter()
            .map(|&slot| cross[slot] - layered.cross_size[slot] * 0.5)
            .fold(f32::MAX, f32::min)
            - padding;
        let interval_max = members
            .iter()
            .map(|&slot| cross[slot] + layered.cross_size[slot] * 0.5)
            .fold(f32::MIN, f32::max)
            + padding;

        for rank in low_rank..=high_rank {
            let slots = layered.order[rank].clone();
            for slot in slots {
                if members.contains(&slot) || matches!(layered.kind[slot], SlotKind::Virtual(_)) {
                    continue;
                }
                let half = layered.cross_size[slot] * 0.5;
                let slot_min = cross[slot] - half;
                let slot_max = cross[slot] + half;
                if slot_max <= interval_min || slot_min >= interval_max {
                    continue;
                }
                let push_right = (interval_max - slot_min) <= (slot_max - interval_min);
                let anchor = cross[slot];
                let delta = if push_right {
                    interval_max - slot_min
                } else {
                    -(slot_max - interval_min)
                };
                for &other in &layered.order[rank] {
                    if members.contains(&other) {
                        continue;
                    }
                    let affected = if push_right {
                        cross[other] >= anchor
                    } else {
                        cross[other] <= anchor
                    };
                    if affected {
                        cross[other] += delta;
                    }
                }
            }
        }
    }
}

fn resolve_overlaps(layered: &Layered, input: &GraphInput, rank: usize, cross: &mut [f32]) {
    let slots = &layered.order[rank];
    for index in 1..slots.len() {
        let previous = slots[index - 1];
        let current = slots[index];
        let minimum = cross[previous]
            + layered.cross_size[previous] * 0.5
            + layered.cross_size[current] * 0.5
            + gap_between(layered, input, previous, current);
        if cross[current] < minimum {
            cross[current] = minimum;
        }
    }
    for index in (0..slots.len().saturating_sub(1)).rev() {
        let current = slots[index];
        let next = slots[index + 1];
        let maximum = cross[next]
            - layered.cross_size[next] * 0.5
            - layered.cross_size[current] * 0.5
            - gap_between(layered, input, current, next);
        if cross[current] > maximum {
            cross[current] = maximum;
        }
    }
}

fn gap_between(layered: &Layered, input: &GraphInput, left: usize, right: usize) -> f32 {
    let both_virtual = matches!(layered.kind[left], SlotKind::Virtual(_))
        && matches!(layered.kind[right], SlotKind::Virtual(_));
    if both_virtual {
        input.edge_lane.max(input.sibling_gap * 0.4)
    } else {
        input.sibling_gap
    }
}

fn assign_rank_positions(layered: &Layered, input: &GraphInput) -> (Vec<f32>, Vec<f32>) {
    let rank_count = layered.order.len();
    let mut extents = vec![0.0f32; rank_count];
    for slot in 0..layered.kind.len() {
        let rank = layered.rank[slot];
        extents[rank] = extents[rank].max(layered.rank_size[slot]);
    }
    let mut offsets = vec![0.0f32; rank_count];
    let mut cursor = 0.0;
    for rank in 0..rank_count {
        offsets[rank] = cursor;
        cursor += extents[rank] + input.rank_gap;
    }
    if matches!(input.direction, Direction::Up | Direction::Left) {
        let total = cursor - input.rank_gap;
        for rank in 0..rank_count {
            offsets[rank] = total - offsets[rank] - extents[rank];
        }
    }
    (offsets, extents)
}

fn split_size(size: Vec2, direction: Direction) -> (f32, f32) {
    match direction {
        Direction::Down | Direction::Up => (size.x, size.y),
        Direction::Right | Direction::Left => (size.y, size.x),
    }
}

fn combine(cross: f32, along: f32, direction: Direction) -> Vec2 {
    match direction {
        Direction::Down | Direction::Up => vec2(cross, along),
        Direction::Right | Direction::Left => vec2(along, cross),
    }
}

fn combine_point(cross: f32, along: f32, direction: Direction) -> Vec2 {
    combine(cross, along, direction)
}
