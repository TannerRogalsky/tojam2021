mod trimesh;
pub mod window;

use glutin::event::{ElementState, MouseButton, VirtualKeyCode};
use rapier3d::dynamics::RigidBodyBuilder;
use rapier3d::geometry::{ColliderBuilder, ColliderHandle};
use rapier3d::na::{UnitQuaternion, Vector3};
use solstice_2d::{
    solstice::{self, Context},
    Color, Draw, Transform3D,
};

pub enum MouseEvent {
    Button(ElementState, MouseButton),
    Moved(f32, f32),
}

#[derive(Default)]
struct InputState {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    prev_mouse_position: (f32, f32),
    mouse_position: (f32, f32),
}

pub struct Resources {
    pub debug_font_data: Vec<u8>,
}

pub struct Game {
    csg: rscsg::dim3::Csg,
    geometry: solstice::mesh::VertexMesh<solstice_2d::Vertex3D>,
    capsule: solstice::mesh::IndexedMesh<solstice_2d::Vertex3D, u32>,
    physics: physics::PhysicsContext,
    ctx: Context,
    gfx: solstice_2d::Graphics,
    debug_font_id: solstice_2d::FontId,
    shader: solstice_2d::Shader,
    camera: camera::CameraState,
    input_state: InputState,

    ground_handle: ColliderHandle,
    capsule_handle: ColliderHandle,
}

impl Game {
    pub fn new(mut ctx: Context, width: f32, height: f32, rsrcs: Resources) -> eyre::Result<Self> {
        let mut physics = physics::PhysicsContext::new(0., -9.81, 0.);

        let csg = rscsg::dim3::Csg::cube(rscsg::dim3::Vector(30., 0.25, 30.), true);
        let ground_handle = physics.add_csg(RigidBodyBuilder::new_static().build(), &csg);
        let vertices = csg.iter_triangles().flat_map(to_vert).collect::<Vec<_>>();
        let geometry = solstice::mesh::VertexMesh::with_data(&mut ctx, &vertices)?;

        let (capsule_handle, capsule) = {
            let coll = ColliderBuilder::capsule_y(1., 0.5).build();
            let (vertices, indices) = coll.shape().as_capsule().unwrap().to_trimesh(20, 20);

            let vertices = vertices
                .into_iter()
                .map(|vertex| {
                    let normal = vertex.coords.normalize();
                    solstice_2d::Vertex3D {
                        position: [vertex.x, vertex.y, vertex.z],
                        uv: [0., 0.],
                        color: [1., 1., 1., 1.],
                        normal: [normal.x, normal.y, normal.z],
                    }
                })
                .collect::<Vec<_>>();
            let indices = indices
                .into_iter()
                .flat_map(|i| std::array::IntoIter::new(i))
                .collect::<Vec<_>>();

            // let vertices = csg.iter_triangles().flat_map(to_vert).collect::<Vec<_>>();
            let capsule = solstice::mesh::IndexedMesh::with_data(&mut ctx, &vertices, &indices)?;

            let rb = RigidBodyBuilder::new_dynamic()
                .translation(0., 4., 0.)
                .restrict_rotations(false, false, false)
                .build();
            (physics.add_body(rb, coll), capsule)
        };

        let mut gfx = solstice_2d::Graphics::new(&mut ctx, width, height)?;
        let debug_font_id = gfx.add_font(std::convert::TryInto::try_into(rsrcs.debug_font_data)?);
        let shader = solstice_2d::Shader::with(include_str!("shader.glsl"), &mut ctx)?;

        Ok(Self {
            csg,
            geometry,
            capsule,
            physics,
            ctx,
            gfx,
            debug_font_id,
            shader,
            camera: camera::CameraState::new(),
            input_state: InputState::default(),
            ground_handle,
            capsule_handle,
        })
    }

    pub fn update(&mut self) {
        if let Some(capsule) = self.physics.rigid_body_mut(self.capsule_handle) {
            let mut v = Vector3::zeros();
            let speed = 1.;
            if self.input_state.w {
                v += Vector3::new(0., 0., -speed);
            }
            if self.input_state.a {
                v += Vector3::new(-speed, 0., 0.);
            }
            if self.input_state.s {
                v += Vector3::new(0., 0., speed);
            }
            if self.input_state.d {
                v += Vector3::new(speed, 0., 0.);
            }
            let v = self.camera.pivot.transform_vector(&v);
            capsule.apply_impulse(v, true);
        }

        self.physics.step();
        self.camera
            .update(self.physics.collider_position(self.capsule_handle));

        let mut g = self.gfx.lock(&mut self.ctx);
        g.set_camera(self.camera);
        g.clear(Color::new(0., 0., 0., 1.));

        let geometry = solstice::Geometry {
            mesh: &self.geometry,
            draw_range: 0..self.geometry.len(),
            draw_mode: solstice::DrawMode::Triangles,
            instance_count: 1,
        };

        g.set_shader(Some(self.shader.clone()));
        g.draw(geometry);

        if let Some(position) = self.physics.collider_position(self.capsule_handle) {
            g.draw_with_transform(
                solstice::Geometry {
                    mesh: &self.capsule,
                    draw_range: 0..self.capsule.len(),
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: 1,
                },
                iso_into_tx(position),
            );

            g.set_shader(None);
            g.set_camera(Transform3D::default());
            let font_scale = 16.;
            g.print(
                format!("capsule: {:?}", position.translation.vector.data),
                self.debug_font_id,
                16.,
                solstice_2d::Rectangle::new(0., font_scale * 0., 720., 720.),
            );
            g.print(
                format!("camera: {:?}", self.camera.position.translation.vector.data),
                self.debug_font_id,
                16.,
                solstice_2d::Rectangle::new(0., font_scale * 1., 720., 720.),
            );
            g.print(
                format!("camera: {:?}", self.camera.position.rotation.euler_angles()),
                self.debug_font_id,
                16.,
                solstice_2d::Rectangle::new(0., font_scale * 2., 720., 720.),
            );
        }
    }

    pub fn handle_key_event(&mut self, state: ElementState, key_code: VirtualKeyCode) {
        let pressed = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };
        match key_code {
            VirtualKeyCode::W => self.input_state.w = pressed,
            VirtualKeyCode::A => self.input_state.a = pressed,
            VirtualKeyCode::S => self.input_state.s = pressed,
            VirtualKeyCode::D => self.input_state.d = pressed,
            _ => {}
        };
    }

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::Button(_, _) => {}
            MouseEvent::Moved(x, y) => {
                if self.input_state.mouse_position == self.input_state.prev_mouse_position
                    && self.input_state.mouse_position == (0., 0.)
                {
                    self.input_state.prev_mouse_position = (x, y);
                    self.input_state.mouse_position = (x, y);
                } else {
                    self.input_state.prev_mouse_position = self.input_state.mouse_position;
                    self.input_state.mouse_position = (x, y);
                }

                let dx = self.input_state.prev_mouse_position.0 - self.input_state.mouse_position.0;
                // let dy = self.input_state.prev_mouse_position.1 - self.input_state.mouse_position.1;

                let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), dx * 0.01);
                self.camera.pivot *= rot;
            }
        }
    }

    pub fn handle_resize(&mut self, width: f32, height: f32) {
        self.ctx.set_viewport(0, 0, width as _, height as _);
        self.gfx.set_width_height(width, height);
    }
}

fn iso_into_tx(position: &rapier3d::math::Isometry<f32>) -> Transform3D {
    use solstice_2d::Rad;
    let v = &position.translation.vector;
    let (rx, ry, rz) = position.rotation.euler_angles();
    Transform3D::translation(v.x, v.y, v.z) * Transform3D::rotation(Rad(rx), Rad(ry), Rad(rz))
}

mod camera {
    use crate::iso_into_tx;
    use rapier3d::math::Isometry;
    use rapier3d::na::{Translation, UnitQuaternion, Vector3};
    use solstice_2d::Transform3D;

    #[derive(Copy, Clone)]
    pub struct CameraState {
        pub velocity: Vector3<f32>,
        pub position: Isometry<f32>,
        pub pivot: UnitQuaternion<f32>,
    }

    impl CameraState {
        pub fn new() -> Self {
            Self {
                velocity: Vector3::new(0., 0., 0.),
                position: Isometry::from_parts(
                    Translation::identity(),
                    UnitQuaternion::from_axis_angle(
                        &Vector3::x_axis(),
                        -std::f32::consts::FRAC_PI_4,
                    ),
                ),
                pivot: Default::default(),
            }
        }

        pub fn update(&mut self, target: Option<&Isometry<f32>>) {
            if let Some(target) = target {
                let offset = Vector3::new(0., 2., 2.);
                let eye = Isometry::from_parts(target.translation.clone(), self.pivot)
                    .transform_vector(&offset);
                let translation = target.translation.clone() * Translation::from(eye);
                let rot = UnitQuaternion::from_axis_angle(
                    &Vector3::x_axis(),
                    -std::f32::consts::FRAC_PI_4,
                );
                let rotation = self.pivot * rot;
                self.position = Isometry::from_parts(translation, rotation);
            }
        }
    }

    impl Into<Transform3D> for CameraState {
        fn into(self) -> Transform3D {
            iso_into_tx(&self.position.inverse())
        }
    }
}

mod physics {
    use rapier3d::dynamics::{CCDSolver, IntegrationParameters, JointSet, RigidBody, RigidBodySet};
    use rapier3d::geometry::{
        BroadPhase, Collider, ColliderBuilder, ColliderHandle, ColliderSet, NarrowPhase,
    };
    use rapier3d::math::Isometry;
    use rapier3d::na::Vector3;
    use rapier3d::pipeline::PhysicsPipeline;

    pub struct PhysicsContext {
        pipeline: PhysicsPipeline,
        gravity: Vector3<f32>,
        integration_parameters: IntegrationParameters,
        broad_phase: BroadPhase,
        narrow_phase: NarrowPhase,
        bodies: RigidBodySet,
        colliders: ColliderSet,
        joints: JointSet,
        ccd_solver: CCDSolver,
    }

    impl PhysicsContext {
        pub fn new(gx: f32, gy: f32, gz: f32) -> Self {
            Self {
                pipeline: PhysicsPipeline::new(),
                gravity: Vector3::new(gx, gy, gz),
                integration_parameters: Default::default(),
                broad_phase: BroadPhase::new(),
                narrow_phase: NarrowPhase::new(),
                bodies: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                joints: JointSet::new(),
                ccd_solver: CCDSolver::new(),
            }
        }

        pub fn step(&mut self) {
            self.pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.joints,
                &mut self.ccd_solver,
                &(),
                &(),
            );
        }

        pub fn add_body(&mut self, body: RigidBody, collider: Collider) -> ColliderHandle {
            let body = self.bodies.insert(body);
            self.colliders.insert(collider, body, &mut self.bodies)
        }

        pub fn add_csg(&mut self, body: RigidBody, csg: &rscsg::dim3::Csg) -> ColliderHandle {
            let vertices = csg
                .iter_triangles()
                .flat_map(|triangle| {
                    std::array::IntoIter::new(triangle.positions).flat_map(|point| {
                        use rapier3d::math::Point;
                        std::array::IntoIter::new([Point::new(point.0, point.1, point.2)])
                    })
                })
                .collect::<Vec<_>>();
            let indices = csg
                .iter_triangles()
                .enumerate()
                .map(|(i, _triangle)| {
                    let i = i as u32 * 3;
                    [i + 0, i + 1, i + 2]
                })
                .collect::<Vec<[u32; 3]>>();
            let collider = ColliderBuilder::trimesh(vertices, indices).build();
            let parent_handle = self.bodies.insert(body);
            self.colliders
                .insert(collider, parent_handle, &mut self.bodies)
        }

        pub fn collider_position(&self, coll: ColliderHandle) -> Option<&Isometry<f32>> {
            self.colliders.get(coll).map(Collider::position)
        }

        pub fn rigid_body_mut(&mut self, coll: ColliderHandle) -> Option<&mut RigidBody> {
            let collider = self.colliders.get(coll)?;
            let body = self.bodies.get_mut(collider.parent())?;
            Some(body)
        }
    }
}

fn to_vert(triangle: rscsg::dim3::Triangle) -> impl Iterator<Item = solstice_2d::Vertex3D> {
    use rscsg::dim3::*;
    let Vector(nx, ny, nz) = triangle.normal;
    std::array::IntoIter::new(triangle.positions).map(move |position| {
        let Vector(x, y, z) = position;
        solstice_2d::Vertex3D {
            position: [x, y, z],
            uv: [0., 0.],
            color: [1., 1., 1., 1.],
            normal: [nx, ny, nz],
        }
    })
}
