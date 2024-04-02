use nalgebra::*;
use vgc::sRGBA;

use crate::sdk::Character;

use super::*;

#[derive(Default, Debug)]
pub struct StudioModel {
    pub ptr: sdk::Ptr<sdk::CStudioHdr>,
    pub studiohdr_ptr: sdk::Ptr<sdk::studiohdr_t>,
    pub studiohdr: sdk::studiohdr_t,

    pub bones: Vec<sdk::mstudiobone_t>,

    pub hitboxset: sdk::mstudiohitboxset_t,

    pub hitboxes: Vec<sdk::mstudiobbox_t>,

    pub hb_lookup: Vec<i32>,

    pub bone_start: i32,
    pub bone_end1: i32,
    pub bone_end2: i32,

    pub bone_head: i32,
    pub bone_body: i32,
}

impl StudioModel {
    pub fn update(&mut self, api: &mut Api, ptr: sdk::Ptr<sdk::CStudioHdr>) -> bool {
        self.ptr = ptr;

        if self.ptr.is_null() {
            self.hitboxes.clear();
            return false;
        }
        // Sometimes this pointer is garbage...
        // Figure out why, this may cause triggerbot to fail!
        if self.ptr.into_raw() % 8 != 0 {
            return false;
        }

        let Ok(cstudio) = api.vm_read(self.ptr) else {
            return false;
        };
        if self.studiohdr_ptr == cstudio.m_pStudioHdr {
            return true;
        }
        self.studiohdr_ptr = cstudio.m_pStudioHdr;
        let Ok(()) = api.vm_read_into(self.studiohdr_ptr, &mut self.studiohdr) else {
            return false;
        };

        self.bone_head = -1;
        self.bone_body = -1;

        // Read bones
        let numbones = self.studiohdr.numbones as usize;
        if numbones > 256 {
            return false;
        }
        if self.bones.len() != numbones {
            self.bones.resize_with(numbones, Default::default);
            self.hb_lookup.clear();
            self.hb_lookup.resize(numbones, -1);
        }
        let Ok(()) = api.vm_read_into(
            self.studiohdr_ptr.field(self.studiohdr.boneoffset()),
            &mut self.bones[..],
        ) else {
            return false;
        };

        // Read first hitboxset
        // if self.studiohdr.numhitboxsets == 0 {
        // 	return false;
        // }
        let Ok(()) = api.vm_read_into(
            self.studiohdr_ptr.field(self.studiohdr.hitboxsetoffset()),
            &mut self.hitboxset,
        ) else {
            return false;
        };

        // Read hitboxes
        let numhitboxes = self.hitboxset.numhitboxes as usize;
        if numhitboxes > 512 {
            self.hitboxes.clear();
            return false;
        }
        if self.hitboxes.len() != numhitboxes {
            self.hitboxes.resize_with(numhitboxes, Default::default);
        }
        if self.hitboxset.numhitboxes > 0 {
            let Ok(()) = api.vm_read_into(
                self.studiohdr_ptr
                    .field(self.studiohdr.hitboxsetoffset() + self.hitboxset.hitboxoffset()),
                &mut self.hitboxes[..],
            ) else {
                return false;
            };
        }

        // Process hitboxes:
        // * Find the head hitbox bone
        // * Create lookup table bone -> hitbox
        let mut bone_end2 = 0;
        for (i, hb) in self.hitboxes.iter().enumerate() {
            bone_end2 = i32::max(bone_end2, hb.bone as i32 + 1);
            if self.bone_head == -1 {
                if hb.group == sdk::HITGROUP_HEAD {
                    self.bone_head = hb.bone as i32;
                }
            }
            if let Some(lookup) = self.hb_lookup.get_mut(hb.bone as usize) {
                *lookup = i as i32;
            }
        }
        self.bone_end2 = bone_end2;

        // Find the range of bones needed for the spine for optimization
        let mut bone_start = i32::MAX;
        let mut bone_end1 = 0;
        let mut bone_body = 0;
        for bbox in self.spine() {
            bone_start = i32::min(bone_start, bbox.bone as i32);
            bone_end1 = i32::max(bone_end1, bbox.bone as i32 + 1);
            bone_body = bbox.bone as i32;
        }
        self.bone_start = bone_start;
        self.bone_end1 = bone_end1;
        self.bone_body = bone_body;

        return true;
    }
    /// Given a hitbox returns its parent hitbox.
    pub fn parent_hitbox(&self, bbox: &sdk::mstudiobbox_t) -> Option<&sdk::mstudiobbox_t> {
        let mut bone = bbox.bone;
        let mut count = 0;
        loop {
            count += 1;
            if count >= self.bones.len() {
                return None;
            }
            let parent = self.bones.get(bone as usize)?.parent as u16;
            // if bone == parent {
            // 	return None;
            // }
            let parent_idx = parent as i32 - 1;
            if parent_idx <= 0 {
                return None;
            }
            if let Some(bbox) = self
                .hb_lookup
                .get(parent_idx as usize)
                .and_then(|&index| self.hitboxes.get(index as usize))
            {
                return Some(bbox);
            }
            bone = parent;
        }
    }
    /// Starting from the head hitbox, iterate over parent bones returning the hitbox until the origin.
    pub fn spine<'a>(&'a self) -> impl 'a + Clone + Iterator<Item=&'a sdk::mstudiobbox_t> {
        self.hitboxes.iter().take_while(|hb| {
            matches!(
                hb.group,
                sdk::HITGROUP_GENERIC
                    | sdk::HITGROUP_HEAD
                    | sdk::HITGROUP_UPPER_BODY
                    | sdk::HITGROUP_LOWER_BODY
            )
        })
    }

    pub fn get_player_bones(&self, origin: &Point3<f32>, bone_array: &BoneArray, hitbox_map: &HitboxMap) -> PlayerBones {
        let get_pos = |hitbox_map_index: Option<usize>| {
            let Some(hitbox_map_index) = hitbox_map_index else { return None; };
            let Some(bone) = self.hitboxes.get(hitbox_map_index) else { return None; };
            let Some(pos) = bone_array.get_vector3(bone.bone as usize) else { return None; };
            Some(origin + pos)
        };
        let get_bone_pos = |bone_index: Option<usize>| {
            let Some(bone_index) = bone_index else { return None; };
            let Some(pos) = bone_array.get_vector3(bone_index as usize) else { return None; };
            Some(origin + pos)
        };

        PlayerBones {
            head: get_bone_pos(hitbox_map.head),
            neck: get_pos(hitbox_map.neck),
            upper_chest: get_pos(hitbox_map.upper_chest),
            lower_chest: get_pos(hitbox_map.lower_chest),
            stomach: get_pos(hitbox_map.stomach),
            hip: get_pos(hitbox_map.hip),
            left_shoulder: get_pos(hitbox_map.left_shoulder),
            left_elbow: get_pos(hitbox_map.left_elbow),
            left_hand: get_pos(hitbox_map.left_hand),
            right_shoulder: get_pos(hitbox_map.right_shoulder),
            right_elbow: get_pos(hitbox_map.right_elbow),
            right_hand: get_pos(hitbox_map.right_hand),
            left_thigh: get_pos(hitbox_map.left_thigh),
            left_knee: get_pos(hitbox_map.left_knee),
            left_foot: get_pos(hitbox_map.left_foot),
            left_toe: get_pos(hitbox_map.left_toe),
            right_thigh: get_pos(hitbox_map.right_thigh),
            right_knee: get_pos(hitbox_map.right_knee),
            right_foot: get_pos(hitbox_map.right_foot),
            right_toe: get_pos(hitbox_map.right_toe),
        }
    }
    pub fn visualize(&self, api: &mut Api, scope: &str) {
        api.visualize(
            scope,
            xfmt! {
                (<h1>"StudioModel"</h1>)
                (<pre>
                    "CStudioHdr:  "{self.ptr}"\n"
                    "studiohdr_t: "{self.studiohdr_ptr}"\n"
                    "\n"
                    "numhitboxsets: "{self.studiohdr.numhitboxsets}"\n"
                    "hitboxsetindex: "{self.studiohdr.hitboxsetindex}"\n"
                    "numbones:      "{self.studiohdr.numbones}"\n"
                    "boneindex:      "{self.studiohdr.boneindex}"\n"
                    "\n"
                    {self.hitboxset:#?}"\n"
                    "\n"
                    "bone_start: "{self.bone_start}"\n"
                    "bone_end1:  "{self.bone_end1}"\n"
                    "bone_end2:  "{self.bone_end2}"\n"
                    "bone_head:  "{self.bone_head}"\n"
                    "bone_body:  "{self.bone_body}"\n"
                    "\n"
                    "spine: "{fmtools::join(" -> ", self.spine().map(|bbox| bbox.bone))}
                </pre>)
                (<h2>"Bones"</h2>)
                (<pre><table>
                <tr>
                    <th>"index"</th>
                    <th>"parent"</th>
                    <th>"unk"</th>
                </tr>)
                for (index, bone) in (self.bones.iter().enumerate()) {
                    <tr>
                        (<td>"Bone "{index}</td>)
                        (<td>"-> "{bone.parent}</td>)
                        (<td>{bone.unk1}" "{bone.unk2}" "{bone.unk3}</td>)
                    </tr>
                }
                </table></pre>

                (<h2>"Hitboxes"</h2>)
                (<pre><table>
                <tr>
                    <th>"index"</th>
                    <th>"bone"</th>
                    <th>"group"</th>
                    <th>"bbmin"</th>
                    <th>"bbmax"</th>
                </tr>)
                for (index, hbox) in (self.hitboxes.iter().enumerate()) {
                    <tr>
                        (<td>"HB "{index}</td>)
                        (<td>{hbox.bone}</td>)
                        (<td>{hbox.group}</td>)
                        (<td>{hbox.bbmin:?}</td>)
                        (<td>{hbox.bbmax:?}</td>)
                    </tr>
                }
                </table></pre>
            },
        );
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct PlayerBones {
    pub head: Option<Point3<f32>>,
    pub neck: Option<Point3<f32>>,
    pub upper_chest: Option<Point3<f32>>,
    pub lower_chest: Option<Point3<f32>>,
    pub stomach: Option<Point3<f32>>,
    pub hip: Option<Point3<f32>>,
    pub left_shoulder: Option<Point3<f32>>,
    pub left_elbow: Option<Point3<f32>>,
    pub left_hand: Option<Point3<f32>>,
    pub right_shoulder: Option<Point3<f32>>,
    pub right_elbow: Option<Point3<f32>>,
    pub right_hand: Option<Point3<f32>>,
    pub left_thigh: Option<Point3<f32>>,
    pub left_knee: Option<Point3<f32>>,
    pub left_foot: Option<Point3<f32>>,
    pub left_toe: Option<Point3<f32>>,
    pub right_thigh: Option<Point3<f32>>,
    pub right_knee: Option<Point3<f32>>,
    pub right_foot: Option<Point3<f32>>,
    pub right_toe: Option<Point3<f32>>,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct HitboxCollisionCustom {
    pub top_vertical: f32,
    pub top_horizontal: f32,
    pub buttom_vertical: f32,
    pub buttom_horizontal: f32,
    pub factor: f32,
}


#[derive(Default, Copy, Clone, Debug)]
pub struct HitboxCollision {
    pub top_left_front: Point3<f32>,
    pub top_left_back: Point3<f32>,
    pub top_right_front: Point3<f32>,
    pub top_right_back: Point3<f32>,
    pub buttom_left_front: Point3<f32>,
    pub buttom_left_back: Point3<f32>,
    pub buttom_right_front: Point3<f32>,
    pub buttom_right_back: Point3<f32>,
}

impl HitboxCollision {
    pub fn iter_edges(&self) -> impl Iterator<Item=(Point3<f32>, Point3<f32>)> {
        vec![
            (self.top_left_front, self.top_right_front),
            (self.buttom_left_front, self.buttom_right_front),
            (self.top_left_back, self.top_right_back),
            (self.buttom_left_back, self.buttom_right_back),
            (self.top_left_front, self.top_left_back),
            (self.top_right_front, self.top_right_back),
            (self.buttom_left_front, self.buttom_left_back),
            (self.buttom_right_front, self.buttom_right_back),
            (self.top_left_front, self.buttom_left_front),
            (self.top_right_front, self.buttom_right_front),
            (self.top_left_back, self.buttom_left_back),
            (self.top_right_back, self.buttom_right_back),
        ]
            .into_iter()
    }
    // 寻找距离方向向量最近的边
    pub fn find_nearest_edge(&self, camera_position: &Point3<f32>, camera_direction: &Vector3<f32>) -> Option<(Point3<f32>, Point3<f32>, Point3<f32>)> {
        // 初始化最小夹角为一个足够大的值
        let mut min_angle = f32::MAX;
        let mut nearest_edge: Option<(Point3<f32>, Point3<f32>, Point3<f32>)> = None;


        // 遍历立方体的所有边
        for (vertex_i, vertex_j) in self.iter_edges() {
            if vertex_i.is_empty() || vertex_j.is_empty() {
                continue;
            }
            // 计算边上距离摄像机位置最近的点
            let nearest_point = self.nearest_point_on_edge(&vertex_i, &vertex_j, camera_position, camera_direction);

            // 计算摄像机位置的方向向量与最近点的夹角
            let direction_to_nearest_point = nearest_point - *camera_position;
            let angle = camera_direction.angle(&direction_to_nearest_point);

            // 更新最小夹角和最近边
            if angle < min_angle {
                min_angle = angle;
                nearest_edge = Some((vertex_i, vertex_j, nearest_point));
            }
        }

        nearest_edge
    }

    // 计算边上距离摄像机位置最近的点
    fn nearest_point_on_edge(&self, vertex_i: &Point3<f32>, vertex_j: &Point3<f32>, camera_position: &Point3<f32>, camera_direction: &Vector3<f32>) -> Point3<f32> {
        // 将线段 AB 转换为向量形式
        let segment_vector = vertex_j - vertex_i;

        // 计算 t 值，它是线段上从起始点到最近点的比例
        // 使用点积和向量长度来避免显式的角度计算
        let t = (segment_vector.dot(&(camera_position - vertex_i)) - segment_vector.dot(&camera_direction) * camera_direction.dot(&(camera_position - vertex_i))) / (segment_vector.norm_squared() - segment_vector.dot(&camera_direction).powi(2));

        // 确保 t 在 0 和 1 之间，以保证点在线段上
        let t = t.max(0.0).min(1.0);

        // 计算并返回最近点
        vertex_i + segment_vector * t
    }


    pub fn update(&mut self, lhs: &Point3<f32>, rhs: &Point3<f32>, custom: &HitboxCollisionCustom) {
        let link_vector = lhs - rhs;
        let ref_vector: OVector<f32, U3> = Vector3::z();
        let rotation_matrix = Rotation3::from_euler_angles(0.0, 0.0, 0.0);

        let horizontal = rotation_matrix * link_vector.cross(&ref_vector);
        let horizontal = horizontal.normalize() * custom.factor;

        let vertical = rotation_matrix * link_vector.cross(&horizontal);
        let vertical = vertical.normalize() * custom.factor;

        let top_horizontal = horizontal * custom.top_horizontal;
        let top_vertical = vertical * custom.top_vertical;

        let buttom_horizontal = horizontal * custom.buttom_horizontal;
        let buttom_vertical = vertical * custom.buttom_vertical;


        self.assign_point(lhs, rhs, top_horizontal, top_vertical, buttom_horizontal, buttom_vertical);
    }
    pub fn update_middle(&mut self, lhs: &Point3<f32>, rhs: &Point3<f32>, custom: &HitboxCollisionCustom) {
        let link_vector = lhs - rhs;
        let ref_vector = Vector3::z();
        let rotation_matrix = Rotation3::from_euler_angles(0.0, 0.0, 0.0);

        let horizontal = rotation_matrix * link_vector.cross(&ref_vector);
        let horizontal = horizontal.normalize() * custom.factor;

        let vertical = rotation_matrix * link_vector.cross(&horizontal);
        let vertical = vertical.normalize() * custom.factor;

        let top_horizontal = horizontal * custom.top_horizontal;
        let top_vertical = vertical * custom.top_vertical;

        let buttom_horizontal = horizontal * custom.buttom_horizontal;
        let buttom_vertical = vertical * custom.buttom_vertical;


        self.assign_point(lhs, rhs, top_horizontal, top_vertical, buttom_horizontal, buttom_vertical);
    }

    fn assign_point(&mut self, lhs: &Point3<f32>, rhs: &Point3<f32>, top_horizontal: OMatrix<f32, Const<3>, U1>, top_vertical: OMatrix<f32, Const<3>, U1>, buttom_horizontal: OMatrix<f32, Const<3>, U1>, buttom_vertical: OMatrix<f32, Const<3>, U1>) {
        self.top_left_front = lhs - top_horizontal + top_vertical;
        self.top_left_back = lhs - top_horizontal - top_vertical;
        self.top_right_front = lhs + top_horizontal + top_vertical;
        self.top_right_back = lhs + top_horizontal - top_vertical;

        self.buttom_left_front = rhs - buttom_horizontal + buttom_vertical;
        self.buttom_left_back = rhs - buttom_horizontal - buttom_vertical;
        self.buttom_right_front = rhs + buttom_horizontal + buttom_vertical;
        self.buttom_right_back = rhs + buttom_horizontal - buttom_vertical;
    }

    pub fn get_pos_draw(&self) -> Vec<([[f32; 3]; 2], sRGBA)> {
        let mut tmp = Vec::new();
        let vertical = sRGBA(200, 20, 20, 200);
        let horizontal = sRGBA(20, 200, 20, 200);
        let depth = sRGBA(20, 20, 200, 200);

        tmp.push(([self.top_left_front.into(), self.top_left_back.into()], vertical));
        tmp.push(([self.top_right_front.into(), self.top_right_back.into()], vertical));
        tmp.push(([self.buttom_left_front.into(), self.buttom_left_back.into()], vertical));
        tmp.push(([self.buttom_right_front.into(), self.buttom_right_back.into()], vertical));

        tmp.push(([self.top_left_front.into(), self.top_right_front.into()], horizontal));
        tmp.push(([self.top_left_back.into(), self.top_right_back.into()], horizontal));
        tmp.push(([self.buttom_left_front.into(), self.buttom_right_front.into()], horizontal));
        tmp.push(([self.buttom_left_back.into(), self.buttom_right_back.into()], horizontal));

        tmp.push(([self.top_left_front.into(), self.buttom_left_front.into()], depth));
        tmp.push(([self.top_left_back.into(), self.buttom_left_back.into()], depth));
        tmp.push(([self.top_right_front.into(), self.buttom_right_front.into()], depth));
        tmp.push(([self.top_right_back.into(), self.buttom_right_back.into()], depth));

        return tmp;
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct HitboxNode {
    pub collision: Option<HitboxCollision>,
    pub custom: HitboxCollisionCustom,
}

impl HitboxNode {
    pub fn update(&mut self, lhs: &Option<Point3<f32>>, rhs: &Option<Point3<f32>>, hitbox_collision_custom: &HitboxCollisionCustom) {
        match (lhs, rhs) {
            (Some(lhs_val), Some(rhs_val)) => {
                let mut collision = HitboxCollision::default();
                collision.update(lhs_val, rhs_val, hitbox_collision_custom);
                self.collision = Some(collision);
            }
            _ => self.collision = None,
        }
    }
    pub fn update_middle(&mut self, lhs: &Option<Point3<f32>>, rhs: &Option<Point3<f32>>, hitbox_collision_custom: &HitboxCollisionCustom) {
        match (lhs, rhs) {
            (Some(lhs_val), Some(rhs_val)) => {
                let mut collision = HitboxCollision::default();
                collision.update_middle(lhs_val, rhs_val, hitbox_collision_custom);
                self.collision = Some(collision);
            }
            _ => self.collision = None,
        }
    }

    pub fn get_pos(&self) -> Option<Vec<([[f32; 3]; 2], sRGBA)>> {
        if let Some(collision) = self.collision {
            Some(collision.get_pos_draw())
        } else { None }
    }
}


#[derive(Default, Copy, Clone, Debug)]
pub struct HitboxNodes {
    pub head: HitboxNode,
    pub neck_upper_chest: HitboxNode,
    pub upper_chest_lower_chest: HitboxNode,
    pub lower_chest_stomach: HitboxNode,
    pub stomach_hip: HitboxNode,

    pub upper_chest_left_shoulder: HitboxNode,
    pub left_shoulder_left_elbow: HitboxNode,
    pub left_elbow_left_hand: HitboxNode,

    // pub upper_chest_right_shoulder: HitboxNode,
    pub right_shoulder_right_elbow: HitboxNode,
    pub right_elbow_right_hand: HitboxNode,

    // pub hip_left_thigh: HitboxNode,
    pub left_thigh_left_knee: HitboxNode,
    pub left_knee_left_foot: HitboxNode,
    pub left_foot_left_toe: HitboxNode,

    // pub hip_right_thigh: HitboxNode,
    pub right_thigh_right_knee: HitboxNode,
    pub right_knee_right_foot: HitboxNode,
    pub right_foot_right_toe: HitboxNode,
}

impl HitboxNodes {
    // 寻找距离方向向量最近的边
    pub fn find_nearest_edge(&self, camera_position: &Point3<f32>, camera_direction: &Vector3<f32>) -> Option<(Point3<f32>, Point3<f32>, Point3<f32>)> {
        // 初始化最小夹角为一个足够大的值
        let mut min_angle = f32::MAX;
        let mut nearest_edge: Option<(Point3<f32>, Point3<f32>, Point3<f32>)> = None;

        // 遍历所有的 HitboxNode
        for node in self.iter() {
            let Some(collision) = node.collision else { continue; };
            // 找到该节点中距离方向向量最近的边
            if let Some((vertex_i, vertex_j, nearest_point)) = collision.find_nearest_edge(camera_position, camera_direction) {
                // 计算边上的两个顶点的夹角
                let direction_to_nearest_point_i = vertex_i - *camera_position;
                let angle_i = camera_direction.angle(&direction_to_nearest_point_i);

                let direction_to_nearest_point_j = vertex_j - *camera_position;
                let angle_j = camera_direction.angle(&direction_to_nearest_point_j);

                // 更新最小夹角和最近边
                if angle_i < min_angle {
                    min_angle = angle_i;
                    nearest_edge = Some((vertex_i, vertex_j, nearest_point));
                }
                if angle_j < min_angle {
                    min_angle = angle_j;
                    nearest_edge = Some((vertex_i, vertex_j, nearest_point));
                }
            }
        }

        nearest_edge
    }
    pub fn find_body_edge(&self, camera_position: &Point3<f32>, camera_direction: &Vector3<f32>) -> Option<(Point3<f32>, Point3<f32>, Point3<f32>)> {
        // 初始化最小夹角为一个足够大的值
        let mut min_angle = f32::MIN;
        let mut nearest_edge: Option<(Point3<f32>, Point3<f32>, Point3<f32>)> = None;

        // 遍历所有的 HitboxNode
        for node in self.iter_body() {
            let Some(collision) = node.collision else { continue; };
            // 找到该节点中距离方向向量最近的边
            if let Some((vertex_i, vertex_j, nearest_point)) = collision.find_nearest_edge(camera_position, camera_direction) {
                // 计算边上的两个顶点的夹角
                let direction_to_nearest_point_i = vertex_i - *camera_position;
                let angle_i = camera_direction.angle(&direction_to_nearest_point_i);

                let direction_to_nearest_point_j = vertex_j - *camera_position;
                let angle_j = camera_direction.angle(&direction_to_nearest_point_j);

                // 更新最小夹角和最近边
                if angle_i > min_angle {
                    min_angle = angle_i;
                    nearest_edge = Some((vertex_i, vertex_j, nearest_point));
                }
                if angle_j > min_angle {
                    min_angle = angle_j;
                    nearest_edge = Some((vertex_i, vertex_j, nearest_point));
                }
            }
        }

        nearest_edge
    }

    pub fn iter(&self) -> impl Iterator<Item=&HitboxNode> {
        vec![
            &self.head,
            // &self.head_neck,
            &self.neck_upper_chest,
            &self.upper_chest_lower_chest,
            &self.lower_chest_stomach,
            // Uncomment these as you implement them
            &self.stomach_hip,
            // &self.upper_chest_left_shoulder,
            &self.left_shoulder_left_elbow,
            &self.left_elbow_left_hand,
            // &self.upper_chest_right_shoulder,
            &self.right_shoulder_right_elbow,
            &self.right_elbow_right_hand,
            // &self.hip_left_thigh,
            &self.left_thigh_left_knee,
            &self.left_knee_left_foot,
            &self.left_foot_left_toe,
            // &self.hip_right_thigh,
            &self.right_thigh_right_knee,
            &self.right_knee_right_foot,
            &self.right_foot_right_toe,
        ]
            .into_iter()
    }

    pub fn iter_body(&self) -> impl Iterator<Item=&HitboxNode> {
        vec![
            &self.head,
            // &self.head_neck,
            &self.neck_upper_chest,
        ]
            .into_iter()
    }

    pub fn update(&mut self, bone_nodes: &PlayerBones) {
        let custom = &HitboxCollisionCustom {
            top_vertical: 2.0,
            top_horizontal: 2.0,
            buttom_vertical: 2.0,
            buttom_horizontal: 2.0,
            factor: 1.0,
        };
        self.head.update_middle(&bone_nodes.head, &bone_nodes.neck, custom);
        // self.head_neck.update(&bone_nodes.head, &bone_nodes.neck, custom);
        self.neck_upper_chest.update(&bone_nodes.neck, &bone_nodes.upper_chest, custom);
        self.upper_chest_lower_chest.update(&bone_nodes.upper_chest, &bone_nodes.lower_chest, custom);
        self.lower_chest_stomach.update(&bone_nodes.lower_chest, &bone_nodes.stomach, custom);

        // self.upper_chest_left_shoulder.update(&bone_nodes.upper_chest, &bone_nodes.left_shoulder, custom);
        self.left_shoulder_left_elbow.update(&bone_nodes.left_shoulder, &bone_nodes.left_elbow, custom);
        self.left_elbow_left_hand.update(&bone_nodes.left_elbow, &bone_nodes.left_hand, custom);

        // self.upper_chest_right_shoulder.update(&bone_nodes.upper_chest, &bone_nodes.right_shoulder, custom);
        self.right_shoulder_right_elbow.update(&bone_nodes.right_shoulder, &bone_nodes.right_elbow, custom);
        self.right_elbow_right_hand.update(&bone_nodes.right_elbow, &bone_nodes.right_hand, custom);

        // self.hip_left_thigh.update(&bone_nodes.hip, &bone_nodes.left_thigh, custom);
        self.left_thigh_left_knee.update(&bone_nodes.left_thigh, &bone_nodes.left_knee, custom);
        self.left_knee_left_foot.update(&bone_nodes.left_knee, &bone_nodes.left_foot, custom);
        self.left_foot_left_toe.update(&bone_nodes.left_foot, &bone_nodes.left_toe, custom);

        // self.hip_right_thigh.update(&bone_nodes.hip, &bone_nodes.right_thigh, custom);
        self.right_thigh_right_knee.update(&bone_nodes.right_thigh, &bone_nodes.right_knee, custom);
        self.right_knee_right_foot.update(&bone_nodes.right_knee, &bone_nodes.right_foot, custom);
        self.right_foot_right_toe.update(&bone_nodes.right_foot, &bone_nodes.right_toe, custom);
    }

    pub fn get_pos(&self) -> Vec<([[f32; 3]; 2], sRGBA)> {
        let mut tmp = Vec::new();

        let append_tmp = |tmp: &mut Vec<([[f32; 3]; 2], sRGBA)>, node: HitboxNode| {
            if let Some(mut draw) = node.get_pos() {
                tmp.append(&mut draw)
            }
        };

        append_tmp(&mut tmp, self.head);
        // tmp.append(&mut self.head_neck.get_pos());
        append_tmp(&mut tmp, self.neck_upper_chest);
        append_tmp(&mut tmp, self.upper_chest_lower_chest);
        append_tmp(&mut tmp, self.lower_chest_stomach);
        // tmp.append(&mut self.upper_chest_left_shoulder.get_pos());
        append_tmp(&mut tmp, self.left_shoulder_left_elbow);
        append_tmp(&mut tmp, self.left_elbow_left_hand);
        // tmp.append(&mut self.upper_chest_right_shoulder.get_pos());
        append_tmp(&mut tmp, self.right_shoulder_right_elbow);
        append_tmp(&mut tmp, self.right_elbow_right_hand);
        append_tmp(&mut tmp, self.left_thigh_left_knee);
        append_tmp(&mut tmp, self.left_knee_left_foot);
        append_tmp(&mut tmp, self.left_foot_left_toe);
        // tmp.append(&mut self.hip_left_thigh.get_pos());
        append_tmp(&mut tmp, self.right_thigh_right_knee);
        append_tmp(&mut tmp, self.right_knee_right_foot);
        append_tmp(&mut tmp, self.right_foot_right_toe);
        // tmp.append(&mut self.hip_right_thigh.get_pos());
        return tmp;
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct HitboxMap {
    pub head: Option<usize>,
    pub neck: Option<usize>,
    pub upper_chest: Option<usize>,
    pub lower_chest: Option<usize>,
    pub stomach: Option<usize>,
    pub hip: Option<usize>,
    pub left_shoulder: Option<usize>,
    pub left_elbow: Option<usize>,
    pub left_hand: Option<usize>,
    pub right_shoulder: Option<usize>,
    pub right_elbow: Option<usize>,
    pub right_hand: Option<usize>,
    pub left_thigh: Option<usize>,
    pub left_knee: Option<usize>,
    pub left_foot: Option<usize>,
    pub left_toe: Option<usize>,
    pub right_thigh: Option<usize>,
    pub right_knee: Option<usize>,
    pub right_foot: Option<usize>,
    pub right_toe: Option<usize>,
}

impl HitboxMap {
    pub fn get_by_model_name(character: &Character) -> Self {
        match character {
            Character::Bloodhound => { Self::Bloundhound }
            Character::Vantage => { Self { head: Some(92), ..Self::Dummie } }
            Character::Conduit => { Self { head: Some(64), ..Self::Dummie } }
            Character::Ash => { Self { left_toe: None, right_toe: None, ..Self::Dummie } }
            Character::Pathfinder => { Self { left_toe: None, right_toe: None, ..Self::Dummie } }
            _ => Self::Dummie
        }
    }


    const Bloundhound: Self = Self {
        head: Some(12),
        neck: Some(0),
        upper_chest: Some(1),
        lower_chest: Some(2),
        stomach: Some(3),
        hip: Some(4),
        left_shoulder: Some(6),
        left_elbow: Some(7),
        left_hand: Some(19),
        right_shoulder: Some(8),
        right_elbow: Some(9),
        right_hand: Some(10),
        left_thigh: Some(20),
        left_knee: Some(12),
        left_foot: Some(13),
        left_toe: Some(14),
        right_thigh: Some(21),
        right_knee: Some(16),
        right_foot: Some(17),
        right_toe: Some(18),
    };

    const Dummie: Self = Self {
        head: Some(12),
        neck: Some(0),
        upper_chest: Some(1),
        lower_chest: Some(2),
        stomach: Some(3),
        hip: Some(4),
        left_shoulder: Some(6),
        left_elbow: Some(7),
        left_hand: Some(8),
        right_shoulder: Some(9),
        right_elbow: Some(10),
        right_hand: Some(11),
        left_thigh: Some(12),
        left_knee: Some(13),
        left_foot: Some(14),
        left_toe: Some(15),
        right_thigh: Some(16),
        right_knee: Some(17),
        right_foot: Some(18),
        right_toe: Some(19),
    };
}