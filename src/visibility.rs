use byondapi::prelude::*;
use byondapi::sys::CByondValue;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

const MAX_RAY_TILES: f32 = 28.0;

#[derive(Clone, Copy)]
pub struct ShadowBox {
    pub id: u32,
    pub z: i32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

struct WorldMap {
    width: usize,
    height: usize,
    depth: usize,
    cells: Vec<u8>,
    boxes: HashMap<u32, ShadowBox>,
}

impl WorldMap {
    fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            depth: 0,
            cells: Vec::new(),
            boxes: HashMap::new(),
        }
    }

    fn init(&mut self, w: usize, h: usize, d: usize) {
        self.width = w;
        self.height = h;
        self.depth = d;
        self.cells = vec![0; w * h * d];
        self.boxes.clear();
    }

    #[inline(always)]
    fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if x < 1 || y < 1 || z < 1 {
            return 0;
        }
        let (xu, yu, zu) = (x as usize, y as usize, z as usize);
        if xu > self.width || yu > self.height || zu > self.depth {
            return 0;
        }
        let idx = (zu - 1) * (self.width * self.height) + (yu - 1) * self.width + (xu - 1);
        self.cells.get(idx).copied().unwrap_or(0)
    }

    #[inline(always)]
    fn set(&mut self, x: usize, y: usize, z: usize, val: u8) {
        if x < 1 || y < 1 || z < 1 || x > self.width || y > self.height || z > self.depth {
            return;
        }
        let idx = (z - 1) * (self.width * self.height) + (y - 1) * self.width + (x - 1);
        if idx < self.cells.len() {
            self.cells[idx] = val;
        }
    }
}

static WORLD_MAP: LazyLock<RwLock<WorldMap>> = LazyLock::new(|| {
    RwLock::new(WorldMap::new())
});

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_map_bounds(argc: u32, argv: *const CByondValue) -> CByondValue {
    if argc >= 3 && !argv.is_null() {
        let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let w = ByondValue(args[0]).get_number().unwrap_or(255.0) as usize;
        let h = ByondValue(args[1]).get_number().unwrap_or(255.0) as usize;
        let d = ByondValue(args[2]).get_number().unwrap_or(1.0) as usize;

        if let Ok(mut map) = WORLD_MAP.write() {
            map.init(w, h, d);
        }
    }
    ByondValue::new_num(1.0).0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_map_cell(argc: u32, argv: *const CByondValue) -> CByondValue {
    if argc >= 4 && !argv.is_null() {
        let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let x = ByondValue(args[0]).get_number().unwrap_or(0.0) as usize;
        let y = ByondValue(args[1]).get_number().unwrap_or(0.0) as usize;
        let z = ByondValue(args[2]).get_number().unwrap_or(0.0) as usize;
        let val = ByondValue(args[3]).get_number().unwrap_or(0.0) as u8;

        if let Ok(mut map) = WORLD_MAP.write() {
            map.set(x, y, z, val);
        }
    }
    ByondValue::new_num(1.0).0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bulk_set_map_cells(argc: u32, argv: *const CByondValue) -> CByondValue {
    if argc >= 1 && !argv.is_null() {
        let raw_val = unsafe { *argv };
        let list_val = ByondValue(raw_val);

        if let Ok(cells_list) = list_val.get_list() {
            if let Ok(mut map) = WORLD_MAP.write() {
                let len = cells_list.len();
                let mut i = 0;
                while i + 3 < len {
                    let x = cells_list[i].get_number().unwrap_or(0.0) as usize;
                    let y = cells_list[i + 1].get_number().unwrap_or(0.0) as usize;
                    let z = cells_list[i + 2].get_number().unwrap_or(0.0) as usize;
                    let val = cells_list[i + 3].get_number().unwrap_or(0.0) as u8;
                    map.set(x, y, z, val);
                    i += 4;
                }
            }
        }
    }
    ByondValue::new_num(1.0).0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_box_world(argc: u32, argv: *const CByondValue) -> CByondValue {
    if argc >= 8 && !argv.is_null() {
        let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let id = ByondValue(args[0]).get_number().unwrap_or(0.0) as u32;
        let z = ByondValue(args[1]).get_number().unwrap_or(1.0) as i32;
        let wx = ByondValue(args[2]).get_number().unwrap_or(0.0);
        let wy = ByondValue(args[3]).get_number().unwrap_or(0.0);

        let px1 = ByondValue(args[4]).get_number().unwrap_or(0.0);
        let py1 = ByondValue(args[5]).get_number().unwrap_or(0.0);
        let px2 = ByondValue(args[6]).get_number().unwrap_or(32.0);
        let py2 = ByondValue(args[7]).get_number().unwrap_or(32.0);

        let x1 = wx + px1.min(px2) / 32.0;
        let y1 = wy + py1.min(py2) / 32.0;
        let x2 = wx + px1.max(px2) / 32.0;
        let y2 = wy + py1.max(py2) / 32.0;

        if let Ok(mut map) = WORLD_MAP.write() {
            map.boxes.insert(id, ShadowBox { id, z, x1, y1, x2, y2 });
        }
    }
    ByondValue::new_num(1.0).0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn remove_box_by_id(argc: u32, argv: *const CByondValue) -> CByondValue {
    if argc >= 1 && !argv.is_null() {
        let raw_val = unsafe { *argv };
        let id = ByondValue(raw_val).get_number().unwrap_or(0.0) as u32;

        if let Ok(mut map) = WORLD_MAP.write() {
            map.boxes.remove(&id);
        }
    }
    ByondValue::new_num(1.0).0
}

#[inline(always)]
fn project_ray_solid(start_x: f32, start_y: f32, px: f32, py: f32) -> (f32, f32) {
    let dx = start_x - px;
    let dy = start_y - py;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.001 {
        return (start_x, start_y);
    }
    let nx = dx / dist;
    let ny = dy / dist;
    (start_x + nx * MAX_RAY_TILES, start_y + ny * MAX_RAY_TILES)
}

fn has_los_to_turf(px: f32, py: f32, target_tx: i32, target_ty: i32, walls: &HashSet<(i32, i32)>) -> bool {
    let tx = target_tx as f32 + 0.5;
    let ty = target_ty as f32 + 0.5;
    let dx = tx - px;
    let dy = ty - py;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.6 {
        return true;
    }

    let steps = (dist * 2.0).ceil() as i32;
    let step_x = dx / (steps as f32);
    let step_y = dy / (steps as f32);
    let mut cx = px;
    let mut cy = py;

    for _ in 1..steps {
        cx += step_x;
        cy += step_y;
        let check_x = cx.floor() as i32;
        let check_y = cy.floor() as i32;

        if walls.contains(&(check_x, check_y)) {
            if check_x != target_tx || check_y != target_ty {
                return false;
            }
        }
    }
    true
}

#[inline(always)]
fn has_los_to_point(px: f32, py: f32, tx: f32, ty: f32, walls: &HashSet<(i32, i32)>) -> bool {
    let dx = tx - px;
    let dy = ty - py;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.3 {
        return true;
    }

    let steps = (dist * 3.0).ceil() as i32;
    let step_x = dx / (steps as f32);
    let step_y = dy / (steps as f32);
    let mut cx = px;
    let mut cy = py;

    for _ in 1..steps {
        cx += step_x;
        cy += step_y;
        if walls.contains(&(cx.floor() as i32, cy.floor() as i32)) {
            return false;
        }
    }
    true
}

#[inline(always)]
fn is_horiz_segment_visible(px: f32, py: f32, x_start: i32, x_end: i32, face_y: f32, walls: &HashSet<(i32, i32)>) -> bool {
    for tx in x_start..=x_end {
        let txf = tx as f32;
        if has_los_to_point(px, py, txf + 0.5, face_y, walls)
            || has_los_to_point(px, py, txf + 0.08, face_y, walls)
            || has_los_to_point(px, py, txf + 0.92, face_y, walls)
        {
            return true;
        }
    }
    false
}

#[inline(always)]
fn is_vert_segment_visible(px: f32, py: f32, y_start: i32, y_end: i32, face_x: f32, walls: &HashSet<(i32, i32)>) -> bool {
    for ty in y_start..=y_end {
        let tyf = ty as f32;
        if has_los_to_point(px, py, face_x, tyf + 0.5, walls)
            || has_los_to_point(px, py, face_x, tyf + 0.08, walls)
            || has_los_to_point(px, py, face_x, tyf + 0.92, walls)
        {
            return true;
        }
    }
    false
}

#[inline(always)]
fn is_tree_point_visible(
    px: f32, py: f32,
    qx: f32, qy: f32,
    walls: &HashSet<(i32, i32)>,
    occluding_trees: &[(f32, f32)],
) -> bool {
    if !has_los_to_point(px, py, qx, qy, walls) {
        return false;
    }

    let qdx = qx - px;
    let qdy = qy - py;
    let q_len_sq = qdx * qdx + qdy * qdy;
    if q_len_sq < 0.001 {
        return true;
    }

    const OCC_R_SQ: f32 = 0.29 * 0.29;
    let max_dist_sq = OCC_R_SQ * q_len_sq;

    for &(ax, ay) in occluding_trees {
        let apx = ax - px;
        let apy = ay - py;
        let dot = apx * qdx + apy * qdy;
        if dot > 0.0 && dot < q_len_sq {
            let cross = apx * qdy - apy * qdx;
            if cross * cross <= max_dist_sq {
                return false;
            }
        }
    }
    true
}

#[inline(always)]
fn make_key(category: u32, x: u32, y: u32, part: u32) -> u32 {
    (category * 600_000) + ((y * 256 + x) * 8) + part
}

#[inline(always)]
fn make_box_key(box_id: u32, part: u32) -> u32 {
    3_000_000 + (box_id * 32) + part
}

#[inline(always)]
fn calc_tri_matrix(
    ax: f32, ay: f32,
    bx: f32, by: f32,
    cx: f32, cy: f32,
) -> [f32; 6] {
    let a = (bx - ax) / 32.0;
    let d = (by - ay) / 32.0;
    let b = (cx - ax) / 32.0;
    let e = (cy - ay) / 32.0;

    let c_mat = ax + 16.0 * a + 16.0 * b - 16.0;
    let f_mat = ay + 16.0 * d + 16.0 * e - 16.0;

    [a, b, c_mat, d, e, f_mat]
}

#[inline(always)]
fn emit_quad(
    emit: &mut impl FnMut(u32, i32, i32, [f32; 6]),
    anchor_x: i32,
    anchor_y: i32,
    w1x: f32, w1y: f32,
    w2x: f32, w2y: f32,
    h1x: f32, h1y: f32,
    h2x: f32, h2y: f32,
    key1: u32,
    key2: u32,
) {
    const OVERLAP_PX: f32 = 2.0;

    let t1_h2 = extend_point(w1x, w1y, h2x, h2y, OVERLAP_PX);
    emit(key1, anchor_x, anchor_y, calc_tri_matrix(w1x, w1y, w2x, w2y, t1_h2.0, t1_h2.1));

    let t2_h1 = extend_point(w1x, w1y, h1x, h1y, OVERLAP_PX);
    emit(key2, anchor_x, anchor_y, calc_tri_matrix(w1x, w1y, h2x, h2y, t2_h1.0, t2_h1.1));
}

#[inline(always)]
fn render_shadow_box(
    emit: &mut impl FnMut(u32, i32, i32, [f32; 6]),
    px: f32, py: f32,
    sbox: &ShadowBox,
) {
    let bx1 = sbox.x1;
    let by1 = sbox.y1;
    let bx2 = sbox.x2;
    let by2 = sbox.y2;

    let width = (bx2 - bx1).max(0.001);
    let height = (by2 - by1).max(0.001);

    let anchor_x = bx1.floor() as i32;
    let anchor_y = by1.floor() as i32;
    let txf = anchor_x as f32;
    let tyf = anchor_y as f32;

    let bid = sbox.id;

    if py < by1 {
        let dy = (by1 - py).max(0.001);
        let t = (height / dy).min(MAX_RAY_TILES);

        let x_exit_l = bx1 + (bx1 - px) * t;
        let x_exit_r = bx2 + (bx2 - px) * t;

        let r1 = project_ray_solid(bx1, by1, px, py);
        let r2 = project_ray_solid(bx2, by1, px, py);

        emit_quad(emit, anchor_x, anchor_y,
            (x_exit_l - txf) * 32.0, (by2 - tyf) * 32.0,
            (x_exit_r - txf) * 32.0, (by2 - tyf) * 32.0,
            (r1.0 - txf) * 32.0, (r1.1 - tyf) * 32.0,
            (r2.0 - txf) * 32.0, (r2.1 - tyf) * 32.0,
            make_box_key(bid, 1), make_box_key(bid, 2));

        if x_exit_l < bx1 {
            emit(make_box_key(bid, 3), anchor_x, anchor_y, calc_tri_matrix(
                (bx1 - txf) * 32.0, (by1 - tyf) * 32.0,
                (bx1 - txf) * 32.0, (by2 - tyf) * 32.0,
                (x_exit_l - txf) * 32.0, (by2 - tyf) * 32.0));
        }

        if x_exit_r > bx2 {
            emit(make_box_key(bid, 4), anchor_x, anchor_y, calc_tri_matrix(
                (bx2 - txf) * 32.0, (by1 - tyf) * 32.0,
                (x_exit_r - txf) * 32.0, (by2 - tyf) * 32.0,
                (bx2 - txf) * 32.0, (by2 - tyf) * 32.0));
        }
    }

    if py > by2 {
        let dy = (py - by2).max(0.001);
        let t = (height / dy).min(MAX_RAY_TILES);

        let x_exit_r = bx2 + (bx2 - px) * t;
        let x_exit_l = bx1 + (bx1 - px) * t;

        let r2 = project_ray_solid(bx2, by2, px, py);
        let r1 = project_ray_solid(bx1, by2, px, py);

        emit_quad(emit, anchor_x, anchor_y,
            (x_exit_r - txf) * 32.0, (by1 - tyf) * 32.0,
            (x_exit_l - txf) * 32.0, (by1 - tyf) * 32.0,
            (r2.0 - txf) * 32.0, (r2.1 - tyf) * 32.0,
            (r1.0 - txf) * 32.0, (r1.1 - tyf) * 32.0,
            make_box_key(bid, 5), make_box_key(bid, 6));

        if x_exit_r > bx2 {
            emit(make_box_key(bid, 7), anchor_x, anchor_y, calc_tri_matrix(
                (bx2 - txf) * 32.0, (by2 - tyf) * 32.0,
                (bx2 - txf) * 32.0, (by1 - tyf) * 32.0,
                (x_exit_r - txf) * 32.0, (by1 - tyf) * 32.0));
        }

        if x_exit_l < bx1 {
            emit(make_box_key(bid, 8), anchor_x, anchor_y, calc_tri_matrix(
                (bx1 - txf) * 32.0, (by2 - tyf) * 32.0,
                (x_exit_l - txf) * 32.0, (by1 - tyf) * 32.0,
                (bx1 - txf) * 32.0, (by1 - tyf) * 32.0));
        }
    }

    if px < bx1 {
        let dx = (bx1 - px).max(0.001);
        let t = (width / dx).min(MAX_RAY_TILES);

        let y_exit_top = by2 + (by2 - py) * t;
        let y_exit_bot = by1 + (by1 - py) * t;

        let r1 = project_ray_solid(bx1, by2, px, py);
        let r2 = project_ray_solid(bx1, by1, px, py);

        emit_quad(emit, anchor_x, anchor_y,
            (bx2 - txf) * 32.0, (y_exit_top - tyf) * 32.0,
            (bx2 - txf) * 32.0, (y_exit_bot - tyf) * 32.0,
            (r1.0 - txf) * 32.0, (r1.1 - tyf) * 32.0,
            (r2.0 - txf) * 32.0, (r2.1 - tyf) * 32.0,
            make_box_key(bid, 9), make_box_key(bid, 10));

        if y_exit_top > by2 {
            emit(make_box_key(bid, 11), anchor_x, anchor_y, calc_tri_matrix(
                (bx1 - txf) * 32.0, (by2 - tyf) * 32.0,
                (bx2 - txf) * 32.0, (by2 - tyf) * 32.0,
                (bx2 - txf) * 32.0, (y_exit_top - tyf) * 32.0));
        }

        if y_exit_bot < by1 {
            emit(make_box_key(bid, 12), anchor_x, anchor_y, calc_tri_matrix(
                (bx1 - txf) * 32.0, (by1 - tyf) * 32.0,
                (bx2 - txf) * 32.0, (y_exit_bot - tyf) * 32.0,
                (bx2 - txf) * 32.0, (by1 - tyf) * 32.0));
        }
    }

    if px > bx2 {
        let dx = (px - bx2).max(0.001);
        let t = (width / dx).min(MAX_RAY_TILES);

        let y_exit_bot = by1 + (by1 - py) * t;
        let y_exit_top = by2 + (by2 - py) * t;

        let r2 = project_ray_solid(bx2, by1, px, py);
        let r1 = project_ray_solid(bx2, by2, px, py);

        emit_quad(emit, anchor_x, anchor_y,
            (bx1 - txf) * 32.0, (y_exit_bot - tyf) * 32.0,
            (bx1 - txf) * 32.0, (y_exit_top - tyf) * 32.0,
            (r2.0 - txf) * 32.0, (r2.1 - tyf) * 32.0,
            (r1.0 - txf) * 32.0, (r1.1 - tyf) * 32.0,
            make_box_key(bid, 13), make_box_key(bid, 14));

        if y_exit_bot < by1 {
            emit(make_box_key(bid, 15), anchor_x, anchor_y, calc_tri_matrix(
                (bx2 - txf) * 32.0, (by1 - tyf) * 32.0,
                (bx1 - txf) * 32.0, (by1 - tyf) * 32.0,
                (bx1 - txf) * 32.0, (y_exit_bot - tyf) * 32.0));
        }

        if y_exit_top > by2 {
            emit(make_box_key(bid, 16), anchor_x, anchor_y, calc_tri_matrix(
                (bx2 - txf) * 32.0, (by2 - tyf) * 32.0,
                (bx1 - txf) * 32.0, (y_exit_top - tyf) * 32.0,
                (bx1 - txf) * 32.0, (by2 - tyf) * 32.0));
        }
    }
}

fn emit_frame_geometry(
    px: f32,
    py: f32,
    radius: f32,
    walls: &HashSet<(i32, i32)>,
    trees_raw: &[(i32, i32, f32, f32)],
    nearby_boxes: &[ShadowBox],
    emit: &mut impl FnMut(u32, i32, i32, [f32; 6]),
) {
    let mut trees: Vec<(i32, i32, f32, f32, f32)> = Vec::with_capacity(trees_raw.len());
    for &(tx, ty, cx, cy) in trees_raw {
        let vx = cx - px;
        let vy = cy - py;
        let dist_sq = vx * vx + vy * vy;
        if dist_sq > 0.35 * 0.35 {
            trees.push((tx, ty, cx, cy, dist_sq));
        }
    }

    trees.sort_unstable_by(|a, b| a.4.total_cmp(&b.4));
    let mut occluding_trees: Vec<(f32, f32)> = Vec::with_capacity(trees.len());

    for &(tx, ty, cx, cy, dist_sq) in &trees {
        let r = 0.35;
        let vx = cx - px;
        let vy = cy - py;
        let dist = dist_sq.sqrt();
        let ux_dir = vx / dist;
        let uy_dir = vy / dist;
        let nx = -uy_dir;
        let ny = ux_dir;

        let t1_x = cx + nx * r + ux_dir * (r * 0.2);
        let t1_y = cy + ny * r + uy_dir * (r * 0.2);
        let t2_x = cx - nx * r + ux_dir * (r * 0.2);
        let t2_y = cy - ny * r + uy_dir * (r * 0.2);

        let visible = is_tree_point_visible(px, py, cx, cy, walls, &occluding_trees)
            || is_tree_point_visible(px, py, t1_x, t1_y, walls, &occluding_trees)
            || is_tree_point_visible(px, py, t2_x, t2_y, walls, &occluding_trees);

        if visible {
            let r1 = project_ray_solid(t1_x, t1_y, px, py);
            let r2 = project_ray_solid(t2_x, t2_y, px, py);

            let txf = tx as f32;
            let tyf = ty as f32;
            let ux = tx as u32;
            let uy = ty as u32;

            emit_quad(emit, tx, ty,
                (t1_x - txf) * 32.0, (t1_y - tyf) * 32.0,
                (t2_x - txf) * 32.0, (t2_y - tyf) * 32.0,
                (r1.0 - txf) * 32.0, (r1.1 - tyf) * 32.0,
                (r2.0 - txf) * 32.0, (r2.1 - tyf) * 32.0,
                make_key(4, ux, uy, 1), make_key(4, ux, uy, 2));

            occluding_trees.push((cx, cy));
        }
    }

    for sbox in nearby_boxes {
        let in_view_x = sbox.x2 >= (px - radius) && sbox.x1 <= (px + radius);
        let in_view_y = sbox.y2 >= (py - radius) && sbox.y1 <= (py + radius);

        if in_view_x && in_view_y {
            let obj_tx = sbox.x1.floor() as i32;
            let obj_ty = sbox.y1.floor() as i32;

            if has_los_to_turf(px, py, obj_tx, obj_ty, walls) {
                render_shadow_box(emit, px, py, sbox);
            }
        }
    }

    for &(tx, ty) in walls {
        let txf = tx as f32;
        let tyf = ty as f32;
        let ux = tx as u32;
        let uy = ty as u32;

        let open_s = !walls.contains(&(tx, ty - 1));
        if open_s && py < tyf {
            let prev_has_s = walls.contains(&(tx - 1, ty)) && !walls.contains(&(tx - 1, ty - 1));
            if !prev_has_s {
                let mut x2 = tx;
                while walls.contains(&(x2 + 1, ty)) && !walls.contains(&(x2 + 1, ty - 1)) {
                    x2 += 1;
                }

                if is_horiz_segment_visible(px, py, tx, x2, tyf - 0.05, walls) {
                    let left_x = tx;
                    let right_x = x2 + 1;
                    let left_xf = left_x as f32;
                    let right_xf = right_x as f32;

                    let dy = (tyf - py).max(0.001);
                    let t = (1.0 / dy).min(MAX_RAY_TILES);

                    let mut x_exit_l = left_xf + (left_xf - px) * t;
                    let mut x_exit_r = right_xf + (right_xf - px) * t;
                    if walls.contains(&(left_x - 1, ty)) { x_exit_l -= 0.02; }
                    if walls.contains(&(x2 + 1, ty)) { x_exit_r += 0.02; }

                    let hit_left = project_ray_solid(left_xf, tyf, px, py);
                    let hit_right = project_ray_solid(right_xf, tyf, px, py);

                    emit_quad(emit, tx, ty,
                        (x_exit_l - txf) * 32.0, 32.0,
                        (x_exit_r - txf) * 32.0, 32.0,
                        (hit_left.0 - txf) * 32.0, (hit_left.1 - tyf) * 32.0,
                        (hit_right.0 - txf) * 32.0, (hit_right.1 - tyf) * 32.0,
                        make_key(0, left_x as u32, uy, 1), make_key(0, left_x as u32, uy, 2));

                    if (left_xf - x_exit_l) * 32.0 > 0.7 {
                        emit(make_key(0, left_x as u32, uy, 3), tx, ty, calc_tri_matrix(
                            (left_xf - txf) * 32.0, 0.0,
                            (left_xf - txf) * 32.0, 32.0,
                            (x_exit_l - txf) * 32.0, 32.0));
                    }

                    if (x_exit_r - right_xf) * 32.0 > 0.7 {
                        emit(make_key(0, left_x as u32, uy, 4), tx, ty, calc_tri_matrix(
                            (right_xf - txf) * 32.0, 0.0,
                            (x_exit_r - txf) * 32.0, 32.0,
                            (right_xf - txf) * 32.0, 32.0));
                    }
                }
            }
        }

        let open_n = !walls.contains(&(tx, ty + 1));
        let tyf_plus_1 = tyf + 1.0;
        if open_n && py > tyf_plus_1 {
            let prev_has_n = walls.contains(&(tx - 1, ty)) && !walls.contains(&(tx - 1, ty + 1));
            if !prev_has_n {
                let mut x2 = tx;
                while walls.contains(&(x2 + 1, ty)) && !walls.contains(&(x2 + 1, ty + 1)) {
                    x2 += 1;
                }

                if is_horiz_segment_visible(px, py, tx, x2, tyf_plus_1 + 0.05, walls) {
                    let left_x = tx;
                    let right_x = x2 + 1;
                    let left_xf = left_x as f32;
                    let right_xf = right_x as f32;

                    let dy = (py - tyf_plus_1).max(0.001);
                    let t = (1.0 / dy).min(MAX_RAY_TILES);

                    let mut x_exit_r = right_xf + (right_xf - px) * t;
                    let mut x_exit_l = left_xf + (left_xf - px) * t;
                    if walls.contains(&(left_x - 1, ty)) { x_exit_l -= 0.02; }
                    if walls.contains(&(x2 + 1, ty)) { x_exit_r += 0.02; }

                    let hit_right = project_ray_solid(right_xf, tyf_plus_1, px, py);
                    let hit_left = project_ray_solid(left_xf, tyf_plus_1, px, py);

                    emit_quad(emit, tx, ty,
                        (x_exit_r - txf) * 32.0, 0.0,
                        (x_exit_l - txf) * 32.0, 0.0,
                        (hit_right.0 - txf) * 32.0, (hit_right.1 - tyf) * 32.0,
                        (hit_left.0 - txf) * 32.0, (hit_left.1 - tyf) * 32.0,
                        make_key(1, left_x as u32, uy, 1), make_key(1, left_x as u32, uy, 2));

                    if (x_exit_r - right_xf) * 32.0 > 0.7 {
                        emit(make_key(1, left_x as u32, uy, 3), tx, ty, calc_tri_matrix(
                            (right_xf - txf) * 32.0, 32.0,
                            (right_xf - txf) * 32.0, 0.0,
                            (x_exit_r - txf) * 32.0, 0.0));
                    }

                    if (left_xf - x_exit_l) * 32.0 > 0.7 {
                        emit(make_key(1, left_x as u32, uy, 4), tx, ty, calc_tri_matrix(
                            (left_xf - txf) * 32.0, 32.0,
                            (x_exit_l - txf) * 32.0, 0.0,
                            (left_xf - txf) * 32.0, 0.0));
                    }
                }
            }
        }

        let open_w = !walls.contains(&(tx - 1, ty));
        if open_w && px < txf {
            let prev_has_w = walls.contains(&(tx, ty - 1)) && !walls.contains(&(tx - 1, ty - 1));
            if !prev_has_w {
                let mut y2 = ty;
                while walls.contains(&(tx, y2 + 1)) && !walls.contains(&(tx - 1, y2 + 1)) {
                    y2 += 1;
                }

                if is_vert_segment_visible(px, py, ty, y2, txf - 0.05, walls) {
                    let bot_y = ty;
                    let top_y = y2 + 1;
                    let bot_yf = bot_y as f32;
                    let top_yf = top_y as f32;

                    let dx = (txf - px).max(0.001);
                    let t = (1.0 / dx).min(MAX_RAY_TILES);

                    let mut y_exit_top = top_yf + (top_yf - py) * t;
                    let mut y_exit_bot = bot_yf + (bot_yf - py) * t;
                    if walls.contains(&(tx, y2 + 1)) { y_exit_top += 0.02; }
                    if walls.contains(&(tx, bot_y - 1)) { y_exit_bot -= 0.02; }

                    let hit_top = project_ray_solid(txf, top_yf, px, py);
                    let hit_bot = project_ray_solid(txf, bot_yf, px, py);

                    emit_quad(emit, tx, ty,
                        32.0, (y_exit_top - tyf) * 32.0,
                        32.0, (y_exit_bot - tyf) * 32.0,
                        (hit_top.0 - txf) * 32.0, (hit_top.1 - tyf) * 32.0,
                        (hit_bot.0 - txf) * 32.0, (hit_bot.1 - tyf) * 32.0,
                        make_key(2, ux, bot_y as u32, 1), make_key(2, ux, bot_y as u32, 2));

                    if (y_exit_top - top_yf) * 32.0 > 0.7 {
                        emit(make_key(2, ux, bot_y as u32, 3), tx, ty, calc_tri_matrix(
                            0.0, (top_yf - tyf) * 32.0,
                            32.0, (y_exit_top - tyf) * 32.0,
                            32.0, (top_yf - tyf) * 32.0));
                    }

                    if (bot_yf - y_exit_bot) * 32.0 > 0.7 {
                        emit(make_key(2, ux, bot_y as u32, 4), tx, ty, calc_tri_matrix(
                            0.0, (bot_yf - tyf) * 32.0,
                            32.0, (bot_yf - tyf) * 32.0,
                            32.0, (y_exit_bot - tyf) * 32.0));
                    }
                }
            }
        }

        let open_e = !walls.contains(&(tx + 1, ty));
        let txf_plus_1 = txf + 1.0;
        if open_e && px > txf_plus_1 {
            let prev_has_e = walls.contains(&(tx, ty - 1)) && !walls.contains(&(tx + 1, ty - 1));
            if !prev_has_e {
                let mut y2 = ty;
                while walls.contains(&(tx, y2 + 1)) && !walls.contains(&(tx + 1, y2 + 1)) {
                    y2 += 1;
                }

                if is_vert_segment_visible(px, py, ty, y2, txf_plus_1 + 0.05, walls) {
                    let bot_y = ty;
                    let top_y = y2 + 1;
                    let bot_yf = bot_y as f32;
                    let top_yf = top_y as f32;

                    let dx = (px - txf_plus_1).max(0.001);
                    let t = (1.0 / dx).min(MAX_RAY_TILES);

                    let mut y_exit_bot = bot_yf + (bot_yf - py) * t;
                    let mut y_exit_top = top_yf + (top_yf - py) * t;
                    if walls.contains(&(tx, y2 + 1)) { y_exit_top += 0.02; }
                    if walls.contains(&(tx, bot_y - 1)) { y_exit_bot -= 0.02; }

                    let hit_bot = project_ray_solid(txf_plus_1, bot_yf, px, py);
                    let hit_top = project_ray_solid(txf_plus_1, top_yf, px, py);

                    emit_quad(emit, tx, ty,
                        0.0, (y_exit_bot - tyf) * 32.0,
                        0.0, (y_exit_top - tyf) * 32.0,
                        (hit_bot.0 - txf) * 32.0, (hit_bot.1 - tyf) * 32.0,
                        (hit_top.0 - txf) * 32.0, (hit_top.1 - tyf) * 32.0,
                        make_key(3, ux, bot_y as u32, 1), make_key(3, ux, bot_y as u32, 2));

                    if (bot_yf - y_exit_bot) * 32.0 > 0.7 {
                        emit(make_key(3, ux, bot_y as u32, 3), tx, ty, calc_tri_matrix(
                            32.0, (bot_yf - tyf) * 32.0,
                            0.0, (y_exit_bot - tyf) * 32.0,
                            0.0, (bot_yf - tyf) * 32.0));
                    }

                    if (y_exit_top - top_yf) * 32.0 > 0.7 {
                        emit(make_key(3, ux, bot_y as u32, 4), tx, ty, calc_tri_matrix(
                            32.0, (top_yf - tyf) * 32.0,
                            0.0, (top_yf - tyf) * 32.0,
                            0.0, (y_exit_top - tyf) * 32.0));
                    }
                }
            }
        }
    }
}

struct TriFrames {
    key: u32,
    anchor_x: i32,
    anchor_y: i32,
    mask: u8,
    mats: [[f32; 6]; 4],
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calculate_shadows_step(argc: u32, argv: *const CByondValue) -> CByondValue {
    let result = std::panic::catch_unwind(|| {
        if argc < 7 || argv.is_null() {
            return ByondValue::null();
        }

        let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let start_px: f32 = ByondValue(args[0]).get_number().unwrap_or(0.0);
        let start_py: f32 = ByondValue(args[1]).get_number().unwrap_or(0.0);
        let end_px: f32 = ByondValue(args[2]).get_number().unwrap_or(0.0);
        let end_py: f32 = ByondValue(args[3]).get_number().unwrap_or(0.0);
        let pz: i32 = ByondValue(args[4]).get_number().unwrap_or(1.0) as i32;
        let radius: f32 = ByondValue(args[5]).get_number().unwrap_or(8.0);
        let num_frames: i32 = ByondValue(args[6]).get_number().unwrap_or(1.0) as i32;

        let map = match WORLD_MAP.read() {
            Ok(m) => m,
            Err(_) => return ByondValue::null(),
        };

        let r_i32 = radius.ceil() as i32;
        let center_x = end_px.floor() as i32;
        let center_y = end_py.floor() as i32;

        let mut walls = HashSet::new();
        let mut trees_raw: Vec<(i32, i32, f32, f32)> = Vec::with_capacity(64);

        for dx in -r_i32..=r_i32 {
            for dy in -r_i32..=r_i32 {
                let tx = center_x + dx;
                let ty = center_y + dy;

                match map.get(tx, ty, pz) {
                    1 => { walls.insert((tx, ty)); }
                    2 => {
                        trees_raw.push((tx, ty, tx as f32 + 0.5, ty as f32 + 0.5));
                    }
                    _ => {}
                }
            }
        }

        let box_r = radius + 1.5;
        let mut nearby_boxes: Vec<ShadowBox> = Vec::with_capacity(16);
        for sbox in map.boxes.values() {
            if sbox.z == pz
                && sbox.x2 >= (end_px - box_r)
                && sbox.x1 <= (end_px + box_r)
                && sbox.y2 >= (end_py - box_r)
                && sbox.y1 <= (end_py + box_r)
            {
                nearby_boxes.push(*sbox);
            }
        }

        let mut out = Vec::with_capacity(if num_frames == 4 { 2048 } else { 512 });

        if num_frames != 4 {
            emit_frame_geometry(end_px, end_py, radius, &walls, &trees_raw, &nearby_boxes,
                &mut |key, ax, ay, m| {
                    out.push(ByondValue::new_num(ax as f32));
                    out.push(ByondValue::new_num(ay as f32));
                    for val in m {
                        out.push(ByondValue::new_num(val));
                    }
                    out.push(ByondValue::new_num(key as f32));
                },
            );
        } else {
            let mut entries: Vec<TriFrames> = Vec::with_capacity(160);
            let mut key_to_idx: HashMap<u32, usize> = HashMap::with_capacity(160);

            for f in 0..4 {
                let progress = (f + 1) as f32 * 0.25;
                let curr_px = start_px + (end_px - start_px) * progress;
                let curr_py = start_py + (end_py - start_py) * progress;

                emit_frame_geometry(curr_px, curr_py, radius, &walls, &trees_raw, &nearby_boxes,
                    &mut |key, ax, ay, m| {
                        if let Some(&idx) = key_to_idx.get(&key) {
                            let entry = &mut entries[idx];
                            entry.mats[f] = m;
                            entry.mask |= 1 << f;
                        } else {
                            let idx = entries.len();
                            key_to_idx.insert(key, idx);
                            let mut mats = [[0.0; 6]; 4];
                            mats[f] = m;
                            entries.push(TriFrames {
                                key,
                                anchor_x: ax,
                                anchor_y: ay,
                                mask: 1 << f,
                                mats,
                            });
                        }
                    },
                );
            }

            for entry in &mut entries {
                if (entry.mask & 0b1000) == 0 {
                    continue;
                }

                let first_f = entry.mask.trailing_zeros() as usize;
                let mut last_valid = entry.mats[first_f];
                for ff in 0..4 {
                    if (entry.mask & (1 << ff)) != 0 {
                        last_valid = entry.mats[ff];
                    } else {
                        entry.mats[ff] = last_valid;
                    }
                }

                out.push(ByondValue::new_num(entry.anchor_x as f32));
                out.push(ByondValue::new_num(entry.anchor_y as f32));
                for ff in 0..4 {
                    for val in entry.mats[ff] {
                        out.push(ByondValue::new_num(val));
                    }
                }
                out.push(ByondValue::new_num(entry.key as f32));
            }
        }

        match ByondValue::new_list() {
            Ok(byond_list) => {
                let _ = byond_list.write_list(&out);
                byond_list
            }
            Err(_) => ByondValue::null(),
        }
    });

    match result {
        Ok(val) => val.0,
        Err(_) => ByondValue::null().0,
    }
}

#[inline(always)]
fn ray_hits_box_strict(px: f32, py: f32, dx: f32, dy: f32, dist: f32, bx1: f32, by1: f32, bx2: f32, by2: f32) -> bool {
    if dist < 0.0001 { return false; }

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    if dx.abs() > 0.0001 {
        let mut tx1 = (bx1 - px) / dx;
        let mut tx2 = (bx2 - px) / dx;
        if tx1 > tx2 { std::mem::swap(&mut tx1, &mut tx2); }
        t_min = t_min.max(tx1);
        t_max = t_max.min(tx2);
    } else if px < bx1 || px > bx2 {
        return false;
    }

    if dy.abs() > 0.0001 {
        let mut ty1 = (by1 - py) / dy;
        let mut ty2 = (by2 - py) / dy;
        if ty1 > ty2 { std::mem::swap(&mut ty1, &mut ty2); }
        t_min = t_min.max(ty1);
        t_max = t_max.min(ty2);
    } else if py < by1 || py > by2 {
        return false;
    }

    if t_min <= 0.0 {
        return false;
    }

    let margin = (0.1_f32 / dist).min(0.49);

    t_min <= t_max && t_max > margin && t_min < (1.0 - margin)
}


#[inline(always)]
fn is_point_visible(px: f32, py: f32, tx: f32, ty: f32, walls: &HashSet<(i32, i32)>) -> bool {
    let dx = tx - px;
    let dy = ty - py;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.0001 { return true; }

    for &(wx, wy) in walls {
        let bx1 = wx as f32;
        let by1 = wy as f32;
        let bx2 = bx1 + 1.0;
        let by2 = by1 + 1.0;
        if ray_hits_box_strict(px, py, dx, dy, dist, bx1, by1, bx2, by2) {
            return false;
        }
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_visible_boxes(argc: u32, argv: *const CByondValue) -> CByondValue {
    let result = std::panic::catch_unwind(|| {
        if argc < 4 || argv.is_null() { return ByondValue::null(); }
        let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let px = ByondValue(args[0]).get_number().unwrap_or(0.0);
        let py = ByondValue(args[1]).get_number().unwrap_or(0.0);
        let pz = ByondValue(args[2]).get_number().unwrap_or(1.0) as i32;
        let radius = ByondValue(args[3]).get_number().unwrap_or(8.0);

        let map = match WORLD_MAP.read() {
            Ok(m) => m,
            Err(_) => return ByondValue::null(),
        };

        let r_i32 = radius.ceil() as i32;
        let center_x = px.floor() as i32;
        let center_y = py.floor() as i32;

        let mut walls = HashSet::new();
        for dx in -r_i32..=r_i32 {
            for dy in -r_i32..=r_i32 {
                let tx = center_x + dx;
                let ty = center_y + dy;
                if map.get(tx, ty, pz) == 1 {
                    walls.insert((tx, ty));
                }
            }
        }

        let check_radius = radius + 2.0;
        let mut nearby_boxes = Vec::new();
        for sbox in map.boxes.values() {
            if sbox.z == pz {
                let in_view_x = sbox.x2 >= (px - check_radius) && sbox.x1 <= (px + check_radius);
                let in_view_y = sbox.y2 >= (py - check_radius) && sbox.y1 <= (py + check_radius);
                if in_view_x && in_view_y {
                    nearby_boxes.push(sbox);
                }
            }
        }

        let mut visible_ids = Vec::new();

        for sbox in &nearby_boxes {
            let in_view_x = sbox.x2 >= (px - radius) && sbox.x1 <= (px + radius);
            let in_view_y = sbox.y2 >= (py - radius) && sbox.y1 <= (py + radius);

            if !in_view_x || !in_view_y {
                continue;
            }

            let cx = (sbox.x1 + sbox.x2) / 2.0;
            let cy = (sbox.y1 + sbox.y2) / 2.0;
            let tx1 = sbox.x1 + (sbox.x2 - sbox.x1) * 0.05;
            let tx2 = sbox.x2 - (sbox.x2 - sbox.x1) * 0.05;
            let ty1 = sbox.y1 + (sbox.y2 - sbox.y1) * 0.05;
            let ty2 = sbox.y2 - (sbox.y2 - sbox.y1) * 0.05;

            let targets = [
                (cx, cy),
                (cx, ty1),
                (cx, ty2),
                (tx1, cy),
                (tx2, cy)
            ];

            let mut occluded_by_prop = true;

            for &(target_x, target_y) in &targets {
                    if !is_point_visible(px, py, target_x, target_y, &walls) {
                        continue;
                    }

                    let dx = target_x - px;
                    let dy = target_y - py;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let mut point_blocked = false;
                    for other_box in &nearby_boxes {
                        if other_box.id == sbox.id { continue; }

                        if ray_hits_box_strict(px, py, dx, dy, dist, other_box.x1, other_box.y1, other_box.x2, other_box.y2) {
                            point_blocked = true;
                            break;
                        }
                    }

                    if !point_blocked {
                        occluded_by_prop = false;
                        break;
                    }
                }

            if !occluded_by_prop {
                visible_ids.push(ByondValue::new_num(sbox.id as f32));
            }
        }

        match ByondValue::new_list() {
            Ok(byond_list) => {
                let _ = byond_list.write_list(&visible_ids);
                byond_list
            },
            Err(_) => ByondValue::null(),
        }
    });

    match result {
        Ok(val) => val.0,
        Err(_) => ByondValue::null().0,
    }
}
#[inline(always)]
fn extend_point(from_x: f32, from_y: f32, to_x: f32, to_y: f32, dist: f32) -> (f32, f32) {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return (to_x, to_y);
    }
    (to_x + (dx / len) * dist, to_y + (dy / len) * dist)
}