use glium;
use prelude::*;
use avec::AVec;
use maths::{Mat4, Point2, Rect};
use core::{blendmodes, BlendMode, rendercontext, RenderContextData, Color, display, Program};
use maths::Vec2;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    position    : [f32; 2],
    offset      : [f32; 2],
    rotation    : f32,
    color       : Color,
    bucket_id   : u32,
    texture_id  : u32,
    texture_uv  : [f32; 2],
    components  : u32,
}
implement_vertex!(Vertex, position, offset, rotation, color, bucket_id, texture_id, texture_uv, components);

/// A wait-free, thread-safe drawing surface for text and sprites.
///
/// In radiant_rs, sprite drawing happens on layers. Layers provide transformation capabilities in
/// the form of model- and view-matrices and the layer's blendmode and color determine
/// how sprites are rendered to the drawing target. Layers can be rendered multiple times using
/// different matrices, blendmodes or colors without having to redraw their contents first.
///
/// For convenience, layers are created with a view-matrix that maps the given dimensions to the
/// entirety of the drawing target. The layer itself is infinite though, and can be transformed at any
/// time before rendering.
///
/// Drawing to a layer is a wait-free atomic operation that can be safely performed from multiple threads at
/// the same time. Modifying layer properties like the matrices may cause other threads to wait.
pub struct Layer {
    view_matrix     : Mutex<Mat4<f32>>,
    model_matrix    : Mutex<Mat4<f32>>,
    blend           : Mutex<BlendMode>,
    color           : Mutex<Color>,
    vertex_data     : AVec<Vertex>,
    vertex_buffer   : Mutex<Option<glium::VertexBuffer<Vertex>>>,
    dirty           : AtomicBool,
    program         : Option<Program>,
}
unsafe impl Send for Layer { }
unsafe impl Sync for Layer { }

impl Layer {

    /// Creates a new layer with given dimensions, meaning that is is created with
    /// a view matrix that maps the given dimensions to the entirety of the drawing target.
    pub fn new<T>(dimensions: T) -> Self where Vec2<f32>: From<T> {
        Self::create(dimensions, None)
    }

    /// Creates a new layer with given dimensions and fragment program.
    pub fn with_program<T>(dimensions: T, program: Program) -> Self where Vec2<f32>: From<T> {
        Self::create(dimensions, Some(program))
    }

    /// Sets a global color multiplicator. Setting this to white means that the layer contents
    /// are renderered in their original colors.
    ///
    /// Note that [`Color`](struct.Color.html)s contain
    /// alpha information and are not clamped to any range, so it is possible to use an overbright
    /// color to brighten the result or use the alpha channel to apply global transparency.
    pub fn set_color(self: &Self, color: Color) -> &Self {
        self.color().set(color);
        self
    }

    /// Returns a mutex guarded mutable reference to the global color multiplicator.
    pub fn color(self: &Self) -> MutexGuard<Color> {
        self.color.lock().unwrap()
    }

    /// Sets the view matrix.
    ///
    /// View matrix transformation is applied after the objects are fully positioned on the layer.
    /// As a result, manipulating the view matrix has the effect of manipulating the layer itself,
    /// e.g. rotating the entire layer.
    pub fn set_view_matrix(self: &Self, matrix: Mat4<f32>) -> &Self {
        self.view_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the view matrix.
    /// See [`set_view_matrix()`](#method.set_view_matrix) for a description of the view matrix.
    pub fn view_matrix(self: &Self) -> MutexGuard<Mat4<f32>> {
        self.view_matrix.lock().unwrap()
    }

    /// Sets the model matrix.
    ///
    /// Model matrix transformation is applied before each object is transformed to its position
    /// on the layer. As a result, manipulating the model matrix has the effect of manipulating
    /// every object on the layer in the same way, e.g. rotating every individual object on the
    /// layer around a point relative to the individual object.
    pub fn set_model_matrix(self: &Self, matrix: Mat4<f32>) -> &Self {
        self.model_matrix().set(matrix);
        self
    }

    /// Returns a mutex guarded mutable reference to the model matrix.
    /// See [`set_model_matrix()`](#method.set_model_matrix) for a description of the model matrix.
    pub fn model_matrix(self: &Self) -> MutexGuard<Mat4<f32>> {
        self.model_matrix.lock().unwrap()
    }

    /// Sets the blendmode.
    pub fn set_blendmode(self: &Self, blendmode: BlendMode) -> &Self {
        self.blendmode().set(blendmode);
        self
    }

    /// Returns a mutex guarded mutable reference to the blendmode.
    pub fn blendmode(self: &Self) -> MutexGuard<BlendMode> {
        self.blend.lock().unwrap()
    }

    /// Removes all previously added object from the layer. Typically invoked after the layer has
    /// been rendered.
    pub fn clear(self: &Self) -> &Self {
        self.dirty.store(true, Ordering::Relaxed);
        self.vertex_data.clear();
        self
    }

    /// Returns the number of sprites the layer can hold without having to perform a blocking reallocation.
    pub fn capacity(self: &Self) -> usize {
        self.vertex_data.capacity() / 4
    }

    /// Returns the number of sprites currently stored the layer.
    pub fn len(self: &Self) -> usize {
        self.vertex_data.len() / 4
    }

    /// Returns the layer wrapped in an std::Arc
    pub fn arc(self: Self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Creates a new layer
    fn create<T>(dimensions: T, program: Option<Program>) -> Self where Vec2<f32>: From<T> {
        let dimensions = Vec2::from(dimensions);
        Layer {
            view_matrix     : Mutex::new(Mat4::viewport(dimensions.0, dimensions.1)),
            model_matrix    : Mutex::new(Mat4::identity()),
            blend           : Mutex::new(blendmodes::ALPHA),
            color           : Mutex::new(Color::white()),
            vertex_data     : AVec::new(rendercontext::INITIAL_CAPACITY * 4),
            vertex_buffer   : Mutex::new(None),
            dirty           : AtomicBool::new(true),
            program         : program,
        }
    }
}

/// Returns a reference to the layer's program, if it has any.
pub fn program(layer: &Layer) -> Option<&Program> {
    layer.program.as_ref()
}

/// Draws a rectangle on given layer.
pub fn add_rect(layer: &Layer, bucket_id: u32, texture_id: u32, components: u32, uv: Rect, pos: Point2, anchor: Point2, dim: Point2, color: Color, rotation: f32, scale: Vec2) {

    layer.dirty.store(true, Ordering::Relaxed);

    // corner positions relative to x/y

    let anchor_x = anchor.0 * dim.0;
    let anchor_y = anchor.1 * dim.1;

    let offset_x0 = -anchor_x * scale.0;
    let offset_x1 = (dim.0 - anchor_x) * scale.0;
    let offset_y0 = -anchor_y * scale.1;
    let offset_y1 = (dim.1 - anchor_y) * scale.1;

    // get vertex_data slice and draw into it

    let map = layer.vertex_data.map(4);

    map.set(0, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x0, offset_y0],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [(uv.0).0, (uv.0).1],
        components  : components,
    });

    map.set(1, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x1, offset_y0],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [(uv.1).0, (uv.0).1],
        components  : components,
    });

    map.set(2, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x0, offset_y1],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [(uv.0).0, (uv.1).1],
        components  : components,
    });

    map.set(3, Vertex {
        position    : [pos.0, pos.1],
        offset      : [offset_x1, offset_y1],
        rotation    : rotation,
        color       : color,
        bucket_id   : bucket_id,
        texture_id  : texture_id,
        texture_uv  : [(uv.1).0, (uv.1).1],
        components  : components,
    });
}

/// Uploads vertex data to the vertex buffer and returns number of vertices uploaded and the mutex-guarded vertex-buffer.
pub fn upload<'a>(layer: &'a Layer, context: &RenderContextData) -> (MutexGuard<'a, Option<glium::VertexBuffer<Vertex>>>, usize) {

    // prepare vertexbuffer if not already done

    let mut vertex_buffer_guard = layer.vertex_buffer.lock().unwrap();

    let num_vertices = {
        let mut vertex_buffer = vertex_buffer_guard.deref_mut();

        // prepare vertexbuffer if not already done

        if vertex_buffer.is_none() {
            *vertex_buffer = Some(glium::VertexBuffer::empty_dynamic(display::handle(&context.display), layer.vertex_data.capacity()).unwrap());
        }

        // copy layer data to vertexbuffer

        if layer.dirty.swap(false, Ordering::Relaxed) {
            let vertex_data = layer.vertex_data.get();
            let num_vertices = vertex_data.len();
            if num_vertices > 0 {
                // resize as neccessary
                if num_vertices > vertex_buffer.as_ref().unwrap().len() {
                    *vertex_buffer = Some(glium::VertexBuffer::empty_dynamic(display::handle(&context.display), layer.vertex_data.capacity()).unwrap());
                }
                // copy data to buffer
                let vb_slice = vertex_buffer.as_ref().unwrap().slice(0 .. num_vertices).unwrap();
                vb_slice.write(&vertex_data[0 .. num_vertices]);
            }
            num_vertices
        } else {
            layer.vertex_data.len()
        }
    };

    (vertex_buffer_guard, num_vertices)
}
