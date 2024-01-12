# Bevy Game of Life Shader Example Part 1

As a fun exercise, I decided to explore using compute shaders using wgpu within bevy. This is part 1 of exploring bevy game of life shader example, and
some ways to enhance the experience.

I assume you have general knowledge of rust, bevy, and wgpu, but I will still explain somethings along the way. If you need a refresher on anything, here
are some helpful resources:

[Learn Rust](https://doc.rust-lang.org/book/)

[Bevy Book](https://bevyengine.org/learn/book/introduction/)

[Bevy Cheatbook](https://bevy-cheatbook.github.io/)

[Learn Wgpu](https://sotrh.github.io/learn-wgpu/)

If you are still ready, lets get started!

## Setup

This tutorial currently follows along using `Bevy 0.10`

I am using the [bevy_game_template](https://github.com/NiklasEi/bevy_game_template) as my starter, just with all the extra code ripped out,
except for main and lib.

```rust
// main.rs

// <imports>
...

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#shader_playground".to_owned()),
                title: "Shader Playground".to_string(),
                present_mode: bevy::window::PresentMode::AutoNoVsync, // unthrottled FPS
                ..default()
            }),
            ..default()
        }))
        .add_plugin(ShaderPlaygroundPlugin)
        .add_system(set_window_icon.on_startup())
        .run();
}

// <window_icon sys>
...

```

```rust
// lib.rs

// <imports>
...

const SIM_SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

pub struct ShaderPlaygroundPlugin;
impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system(window_fps);
    }
}

fn window_fps(diagnostics: Res<Diagnostics>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                window.title = format!("{fps_smoothed:.2}");
            }
        }
    }
}

```

Pretty simple setup. Just added a simple fps system to print the FPS at the top of the window. We need to setup our canvas for our simulation.
We will just use a simple image constrained to our `SIM_SIZE` variable. You can organize your code how you feel, but I will be breaking out the code
from the original example into separate files for readability.

```rust
// image.rs

#[derive(Resource, Clone, Deref, ExtractResource)]
pub struct GameOfLifeImage(pub Handle<Image>);

pub fn create_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image
}

```

This is just a simple helper function to create an image. We will be using this to create our simulation image. `GameOfLifeImage` will hold a reference to our
handle image so that it doesn't get unloaded. You will see it is annotated with `ExtractResource` derive. This is required to extract the image from the main bevy
world into the render world. You can also manually derive this, if needed. Lets use it within our setup function. Back in
`lib.rs`:

```rust
...
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = image::create_image(SIM_SIZE.0, SIM_SIZE.1);
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(image::GameOfLifeImage(image));
}
```

Quick and easy. Just create our image, create a sprite based off the image, spawn a 2D camera, and then inject our resource into bevy world.
Now we just need to hookup our system back in the plugin.

```rust
fn build(&self, app: &mut App) {
    app.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(window_fps);
}
```

Running the simulation should produce a black screen and nothing more. But hey! No errors! Lets move on.

As mentioned earlier, bevy has a render world which is extracted each render instance. We need to tell our render setup how to access members from the main world.
In our plugin, add the `ExtractResourcePlugin` for our `GameOfLifeImage` resource.

```rust
...
// Extract the game of life image resource from the main world into the render world
// for operation on by the compute shader and display on the sprite.
app.add_plugin(ExtractResourcePlugin::<GameOfLifeImage>::default());
...
```

Now onto the actual meat of why I am writing this. Lets get into wgpu. We will be using a compute shader, so our pipeline will need to be a `ComputePipeline`.

> A pipeline describes all the actions the gpu will perform when acting on a set of data

Lets create a new file, `pipeline.rs` and add the following:

```rust
use bevy::{prelude::*, render::render_resource::*};

#[derive(Resource)]
pub struct GameOfLifePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {}
}

```

Our pipeline resource holds the two compute pipeline id's we will be using init (for setup) and update (for each frame). We also need to hold onto
the bind group layout.

> a bind group layout is a way to describe the structure of resources that a shader will access during execution. The resources include buffers, textures, and samplers

Lets expand upon our `FromWorld` implementation.

```rust
impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/game_of_life.wgsl");

        ...
```

WHOA! What is this weird layout stuff? First thing first, the `RenderDevice` is equivalent to `wgpu::Device`. We are just using the bevy wrapper.
I like to add labels to all my shader creations, just to make it easier to track bugs when they do arise. This layout is telling the pipeline that at
binding(0) we expect there to be a binding of type `StorageTexture`, that is read_write, and has the format of `Rgba8Unorm`.

We then pull the pipeline cache to create our pipelines and pull our shader from the asset's folder.

## Compute Shaders

> A compute shader is simply a shader that allows you to leverage the GPU's parallel computing power for arbitrary tasks. You can use them for anything from creating a texture to running a neural network. I'll get more into how they work in a bit, but for now suffice to say that we're going to use them to create the vertex and index buffers for our terrain.
> As of writing, compute shaders are still experimental on the web. You can enable them on beta versions of browsers such as Chrome Canary and Firefox Nightly. Because of this I'll cover a method to use a fragment shader to compute the vertex and index buffers after we cover the compute shader method.

```rust
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        shader: shader.clone(),
        shader_defs: vec![],
        layout: vec![texture_bind_group_layout.clone()],
        entry_point: Cow::from("init"),
        push_constant_ranges: Vec::new(),
        label: Some(std::borrow::Cow::Borrowed("Game of Life Init Pipeline")),
    });
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        shader,
        shader_defs: vec![],
        layout: vec![texture_bind_group_layout.clone()],
        entry_point: Cow::from("update"),
        push_constant_ranges: Vec::new(),
        label: Some(std::borrow::Cow::Borrowed("Game of Life Update Pipeline")),
    });

    GameOfLifePipeline {
        texture_bind_group_layout,
        init_pipeline,
        update_pipeline,
    }
```

The init and update pipeline use the same layout and shader. We are not using push constants yet, so we can leave that as an empty vector, same with shader_defs.
The important part is the entry_point. This is the name of the function in the shader that will be executed.

Lets quickly setup our shader and then we can move onto the actual compute shader.

```glsl
// assets/game_of_life.wgsl

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {}
```

We can see that from what we defined in our `BindGroupLayoutDescriptor` that we define a `texture_storage_2d<rgba8unorm, read_write>` at `binding(0)`. Looking
back at our descriptor:

```rust
BindGroupLayoutDescriptor {
    label: Some("Game of Life Bind Group Layout"),
    entries: &[BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access: StorageTextureAccess::ReadWrite,
            format: TextureFormat::Rgba8Unorm,
            view_dimension: TextureViewDimension::D2,
        },
        count: None,
    }],
}
```

We see that our `BindGroupLayoutEntry` matches what we expect at `group(0) binding(0)`. This has to match 1:1, otherwise wgpu will panic on run.

Afterwards, we simply just adds our entry points that we defined in the pipeline's. The `@workgroup_size` is the size of the workgroup that will be
executed on the gpu. Bevy example uses 8.
Let's add our newly created pipeline resource to the render world. Back in the plugin:

```rust
...
let render_app = app.sub_app_mut(RenderApp);
render_app.init_resource::<GameOfLifePipeline>();
```

Running the example now should produce the same screen, but still no errors. We are getting closer to actually doing something. More in part 2.

Code can be found on github: [Part 1](https://github.com/n16hth4wkk/bevy_shader_playground/tree/explore_part1/sims/game_of_life_sim/src)
