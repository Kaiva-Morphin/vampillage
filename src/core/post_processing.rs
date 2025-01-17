//! This example shows how to create a custom render pass that runs after the main pass
//! and reads the texture generated by the main pass.
//!
//! The example shader is a very simple implementation of chromatic aberration.
//! To adapt this example for 2D, replace all instances of 3D structures (such as `Core3D`, etc.) with their corresponding 2D counterparts.
//!
//! This is a fairly low level example and assumes some familiarity with rendering concepts and wgpu.
use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    }, ecs::query::QueryItem, math::{vec2, vec4}, prelude::*, render::{
        camera::ScalingMode, extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        }, render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        }, render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        }, renderer::{RenderContext, RenderDevice}, texture::BevyDefault, view::ViewTarget, RenderApp
    }, window::WindowResized
};

use super::camera::plugin::MainCamera;

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/post_processing.wgsl";

/// It is generally encouraged to set up post processing effects as a plugin
pub struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // The settings will be a component that lives in the main world but will
            // be extracted to the render world every frame.
            // This makes it possible to control the effect from the main world.
            // This plugin will take care of extracting it automatically.
            // It's important to derive [`ExtractComponent`] on [`PostProcessingSettings`]
            // for this plugin to work correctly.
            ExtractComponentPlugin::<PostProcessUniform>::default(),
            // The settings will also be the data used in the shader.
            // This plugin will prepare the component for the GPU by creating a uniform buffer
            // and writing the data to that buffer every frame.
            UniformComponentPlugin::<PostProcessUniform>::default(),
        ));
        app.add_systems(PostStartup, late_setup);
        app.add_systems(Update, (update_settings, on_resize_system));

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
            // It currently runs on each view/camera and executes each node in the specified order.
            // It will make sure that any node that needs a dependency from another node
            // only runs when that dependency is done.
            //
            // Each node can execute arbitrary work, but it generally runs at least one render pass.
            // A node only has access to the render world, so if you need data from the main world
            // you need to extract it manually or with the plugin like above.
            // Add a [`Node`] to the [`RenderGraph`]
            // The Node needs to impl FromWorld
            //
            // The [`ViewNodeRunner`] is a special [`Node`] that will automatically run the node for each view
            // matching the [`ViewQuery`]
            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(
                // Specify the label of the graph, in this case we want the graph for 3d
                Core2d,
                // It also needs the label of the node
                PostProcessLabel,
            )
            .add_render_graph_edges(
                Core2d,
                // Specify the node ordering.
                // This will automatically create all required node edges to enforce the given ordering.
                (
                    Node2d::Tonemapping,
                    PostProcessLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<PostProcessPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

// The post process node used for the render graph
#[derive(Default)]
struct PostProcessNode;

// The ViewNode trait is required by the ViewNodeRunner
impl ViewNode for PostProcessNode {
    // The node needs a query to gather data from the ECS in order to do its rendering,
    // but it's not a normal system so we need to define it manually.
    //
    // This query will only run on the view entity
    type ViewQuery = (
        &'static ViewTarget,
        // This makes sure the node only runs on cameras with the PostProcessUniform component
        &'static PostProcessUniform,
        // As there could be multiple post processing components sent to the GPU (one per camera),
        // we need to get the index of the one that is associated with the current view.
        &'static DynamicUniformIndex<PostProcessUniform>,
    );

    // Runs the node logic
    // This is where you encode draw commands.
    //
    // This will run on every view on which the graph is running.
    // If you don't want your effect to run on every camera,
    // you'll need to make sure you have a marker component as part of [`ViewQuery`]
    // to identify which camera(s) should run the effect.
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        // Get the settings uniform binding
        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessUniform>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &post_process_pipeline.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                post_process.source,
                // Use the sampler created for the pipeline
                &post_process_pipeline.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(pipeline);
        // By passing in the index of the post process settings on this view, we ensure
        // that in the event that multiple settings were sent to the GPU (as would be the
        // case with multiple cameras), we use the correct one.
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // We need to define the bind group layout used for our pipeline
        let layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::FRAGMENT,
                (
                    // The screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // The sampler that will be used to sample the screen texture
                    sampler(SamplerBindingType::Filtering),
                    // The settings uniform that will control the effect
                    uniform_buffer::<PostProcessUniform>(true),
                ),
            ),
        );

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Get the shader handle
        let shader = world.load_asset(SHADER_ASSET_PATH);

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba16Float, //bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
//#[derive(Default)]
pub struct PostProcessUniform {
    time: f32,
    translation: Vec2,
    target_height: f32,
    target_width: f32,
    pub height: f32,
    pub width: f32,

    pub daytime: f32,
    
    day_color: Vec4,
    night_color: Vec4,

    vignette_strength: f32,

    wave_strength: f32,

    // WebGL2 structs must be 16 byte aligned.
    //[cfg(feature = "webgl2")]
    //webgl2_padding: Vec3,
}

impl Default for PostProcessUniform {
    fn default() -> Self {
        PostProcessUniform{
            daytime: 0.,
            translation: vec2(0., 0.),
            day_color: vec4(1.0, 0.791, 0.396, 1.),
            //day_color: vec4(1.5, 1.1, 0.6, 1.),
            night_color: vec4(0.182, 0.221, 1., 1.),
            //night_color: vec4(0.005, 0.01, 0.03, 1.),
            vignette_strength: 0.35,
            wave_strength: 15.,
            time: 0.,
            
            target_height: 1.,
            target_width: 1.,
            height: 1.,
            width: 1.,
        }
    }
}


fn late_setup(
    mut commands: Commands,
    camera: Query<(&OrthographicProjection, &GlobalTransform, Entity), With<MainCamera>>
) {
    let (projection, t, e) = camera.single();
    let ScalingMode::AutoMin{ min_width: w, min_height: h } = projection.scaling_mode else {return};
    commands.entity(e).insert(
        PostProcessUniform {
            time: 0.0,
            translation: t.translation().xy(),
            target_height: h,
            target_width: w,
            height: h,
            width: w,
            daytime: 0.,
            ..default()
        }
    );
}



fn update_settings(
    mut settings: Query<&mut PostProcessUniform>, 
    camera: Query<&GlobalTransform, With<MainCamera>>,
    time: Res<Time>,    
) {
    let mut settings = settings.single_mut();
    let camera_transform = camera.single().translation().xy();
    settings.translation = camera_transform;
    settings.time = time.elapsed_seconds();
}

fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut settings: Query<&mut PostProcessUniform>,
) {
    for e in resize_reader.read() {
        for mut setting in &mut settings {
            setting.width = e.width;
            setting.height = e.height;
        }
    }
}


