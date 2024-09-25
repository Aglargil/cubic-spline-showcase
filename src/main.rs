//! 这个示例演示了如何使用 `bevy_gizmos` 包在 2D 中绘制线条和点。

use bevy::{color::palettes::css::*, math::Vec2, prelude::*};

#[derive(Default, Resource)]
struct MousePosition(Option<Vec2>);

// 我们可以创建自己的 gizmo 配置组！
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

struct MovablePoint {
    position: Vec2,
    show_size: f32,
    selected_size: f32,
    default_color: Srgba,
    selected_color: Srgba,
    is_selected: bool,
}

#[derive(Default, Resource)]
struct ControlPoints {
    points: Vec<MovablePoint>,
}

impl Default for MovablePoint {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            show_size: 5.0,
            selected_size: 10.0,
            default_color: GREEN,
            selected_color: RED,
            is_selected: false,
        }
    }
}

fn setup(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    commands.spawn(Camera2dBundle::default());
    let (my_config, _) = config_store.config_mut::<MyRoundGizmos>();
    my_config.line_width = 5.0;
}

fn plot_line(mut gizmos: Gizmos, control_points: Res<ControlPoints>) {
    let movable_points: Vec<&MovablePoint> = control_points.points.iter().collect();
    if movable_points.len() < 2 {
        return;
    }
    let points: Vec<Vec2> = movable_points.iter().map(|p| p.position).collect();

    gizmos.linestrip_2d(points.clone(), WHITE);

    // 使用辅助函数渲染 B-Spline
    let b_spline = CubicBSpline::new(points.clone());
    render_curve(&mut gizmos, b_spline.to_curve(), PINK);

    // 使用辅助函数渲染 Cardinal Spline
    let cardinal_spline = CubicCardinalSpline::new_catmull_rom(points.clone());
    render_curve(&mut gizmos, cardinal_spline.to_curve(), YELLOW);

    // 特殊情况：渲染 Bezier Spline
    if points.len() >= 4 {
        let points_array: Vec<[Vec2; 4]> = vec![[points[0], points[1], points[2], points[3]]];
        let bezier_spline = CubicBezier::new(points_array);
        render_curve(&mut gizmos, bezier_spline.to_curve(), GREEN);
    }
}

fn plot_point(mut gizmos: Gizmos<MyRoundGizmos>, control_points: Res<ControlPoints>) {
    for point in control_points.points.iter() {
        let color = if point.is_selected {
            point.selected_color
        } else {
            point.default_color
        };
        gizmos.circle_2d(
            Isometry2d::from_xy(point.position.x, point.position.y),
            point.show_size,
            color,
        );
    }
}

fn move_point_with_mouse(
    mut control_points: ResMut<ControlPoints>,
    input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let mut clear_selected = || {
        for point in control_points.points.iter_mut() {
            point.is_selected = false;
        }
    };
    let Some(mouse_position) = mouse_position.0 else {
        clear_selected();
        return;
    };
    if !input.pressed(MouseButton::Left) {
        clear_selected();
        return;
    }

    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    // Convert the starting point and end point (current mouse pos) into world coords:
    let Ok(mouse_point) = camera.viewport_to_world_2d(camera_transform, mouse_position) else {
        return;
    };
    for point in control_points.points.iter_mut() {
        if point.is_selected {
            point.position = mouse_point;
            return;
        }
    }

    for point in control_points.points.iter_mut() {
        if point.position.distance(mouse_point) < point.selected_size {
            point.is_selected = true;
            break;
        }
    }
}

/// Update the current cursor position and track it in the [`MousePosition`] resource.
fn handle_mouse_move(
    mut cursor_events: EventReader<CursorMoved>,
    mut mouse_position: ResMut<MousePosition>,
) {
    if let Some(cursor_event) = cursor_events.read().last() {
        mouse_position.0 = Some(cursor_event.position);
    }
}

fn add_point_with_right_mouse(
    camera: Query<(&Camera, &GlobalTransform)>,
    input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut control_points: ResMut<ControlPoints>,
) {
    if input.just_pressed(MouseButton::Right) {
        let Some(mouse_position) = mouse_position.0 else {
            return;
        };
        let Ok((camera, camera_transform)) = camera.get_single() else {
            return;
        };
        let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, mouse_position)
        else {
            return;
        };
        control_points.points.push(MovablePoint {
            position: world_position,
            ..default()
        });
    }
}

fn handle_keypress(keyboard: Res<ButtonInput<KeyCode>>, mut control_points: ResMut<ControlPoints>) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        control_points.points.pop();
    }
}

// 辅助函数，用于生成和渲染曲线
fn render_curve<E>(gizmos: &mut Gizmos, curve: Result<CubicCurve<Vec2>, E>, color: Srgba) {
    if let Ok(curve) = curve {
        let resolution = 100 * curve.segments().len(); // 根据曲线段数调整分辨率
        gizmos.linestrip(
            curve.iter_positions(resolution).map(|pt| pt.extend(0.0)),
            color,
        );
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MousePosition::default())
        .insert_resource(ControlPoints::default())
        .init_gizmo_group::<MyRoundGizmos>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_keypress,
                handle_mouse_move,
                move_point_with_mouse,
                add_point_with_right_mouse,
                plot_point,
                plot_line,
            )
                .chain(),
        )
        .run();
}
