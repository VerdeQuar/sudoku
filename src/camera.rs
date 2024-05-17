use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};


#[derive(Resource)]
pub struct CameraControl {
    pub target_pos: Vec3,
    pub target_scale: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            target_pos: Default::default(),
            target_scale: 1.,
        }
    }
}

pub fn movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,

    mut event_wheel: EventReader<MouseWheel>,
    mut event_move: EventReader<MouseMotion>,

    input_mouse: Res<ButtonInput<MouseButton>>,

    mut control: ResMut<CameraControl>,
) {
    let Ok((mut transform, mut projection)) = query.get_single_mut() else {
        return;
    };

    if input_mouse.pressed(MouseButton::Left) {
        for ev in event_move.read() {
            control.target_pos +=
                projection.scale * ev.delta.extend(0.) * time.delta_seconds() * 200. * Vec3::new(-1., 1., 0.);
        }
    }
    if transform.translation.distance_squared(control.target_pos) > 0.01 {
        transform.translation = transform
            .translation
            .lerp(control.target_pos, 40. * time.delta_seconds());
    }

    for ev in event_wheel.read() {
        control.target_scale -= ev.y * 0.02;
        control.target_scale = control.target_scale.max(0.01);
    }

    if (projection.scale - control.target_scale).abs() > 0.01 {
        projection.scale = projection.scale
        + ((control.target_scale - projection.scale) * 20. * time.delta_seconds());
    }

    event_move.clear();
}
