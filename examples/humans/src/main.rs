use kuro::prelude::*;

fn main() {
    let mut app = App::new(AppConfig {
        title: "Kuro Humans 0.1.0".into(),
        width: 1280,
        height: 720,
        msaa: Msaa::X4,
        camera: CameraConfig {
            position: Vec3::new(0.0, 6.0, 14.0),
            target: Vec3::ZERO,
            ..Default::default()
        },
    });

    let tree = app
        .storage
        .load_gltf(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/tree.glb"));

    let t1 = app.workspace.spawn(tree, Vec3::new(-3.0, 0.0, 0.0));
    let t2 = app.workspace.spawn_with(
        tree,
        SpawnConfig {
            position: Vec3::new(3.0, 0.0, -2.0),
            scale: Vec3::splat(1.3),
            color: LinSrgba::new(0.4, 0.9, 0.5, 1.0),
            ..Default::default()
        },
    );

    app.run(move |workspace, dt| {
        if let Some(model) = workspace.get_model_mut(t1) {
            model.rotation *= Quat::from_rotation_y(0.6 * dt);
        }
        if let Some(model) = workspace.get_model_mut(t2) {
            model.rotation *= Quat::from_rotation_y(-0.4 * dt);
        }
    });
}
