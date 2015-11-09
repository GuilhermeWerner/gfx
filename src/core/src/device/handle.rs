// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![deny(missing_docs, missing_copy_implementations)]

//! Device resource handles

use std::cmp;
use std::mem;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use super::{shade, pso, tex, Resources, BufferInfo};


/// Raw (untyped) Buffer Handle
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RawBuffer<R: Resources>(Arc<R::Buffer>, BufferInfo);

impl<R: Resources> RawBuffer<R> {
    /// Get raw buffer info
    pub fn get_info(&self) -> &BufferInfo { &self.1 }

    /// Compare the handle by the reference (not data)
    pub fn cmp_ref(&self, other: &RawBuffer<R>) -> cmp::Ordering {
        self.0.cmp_ref(&other.0)
    }
}

/// Type-safe buffer handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Buffer<R: Resources, T> {
    raw: RawBuffer<R>,
    phantom_t: PhantomData<T>,
}

impl<R: Resources, T> Buffer<R, T> {
    /// Create a type-safe Buffer from a RawBuffer
    pub fn from_raw(handle: RawBuffer<R>) -> Buffer<R, T> {
        Buffer {
            raw: handle,
            phantom_t: PhantomData,
        }
    }

    /// Cast the type this Buffer references
    pub fn cast<U>(self) -> Buffer<R, U> {
        Buffer::from_raw(self.raw)
    }

    /// Get the underlying raw Handle
    pub fn raw(&self) -> &RawBuffer<R> {
        &self.raw
    }

    /// Get the associated information about the buffer
    pub fn get_info(&self) -> &BufferInfo {
        self.raw.get_info()
    }

    /// Get the number of elements in the buffer.
    ///
    /// Fails if `T` is zero-sized.
    pub fn len(&self) -> usize {
        assert!(mem::size_of::<T>() != 0, "Cannot determine the length of zero-sized buffers.");
        self.get_info().size / mem::size_of::<T>()
    }
}

/// Array Buffer Handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ArrayBuffer<R: Resources>(Arc<R::ArrayBuffer>);

/// Shader Handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Shader<R: Resources>(Arc<R::Shader>);

/// Program Handle
#[derive(Clone, Debug, PartialEq)]
pub struct Program<R: Resources>(Arc<R::Program>, shade::ProgramInfo);

impl<R: Resources> Program<R> {
    /// Get program info
    pub fn get_info(&self) -> &shade::ProgramInfo { &self.1 }
}

/// Pipeline State Handle
#[derive(Clone, Debug, PartialEq)]
pub struct PipelineState<R: Resources>(
    Arc<R::PipelineState>,
    pso::PipelineInfo,
    shade::ProgramInfo,
);

impl<R: Resources> PipelineState<R> {
    /// get pipeline info
    pub fn get_info(&self) -> &pso::PipelineInfo { &self.1 }

    /// Get program info
    pub fn get_program_info(&self) -> &shade::ProgramInfo { &self.2 }
}

/// Frame Buffer Handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FrameBuffer<R: Resources>(Arc<R::FrameBuffer>);

/// Surface Handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Surface<R: Resources>(Arc<R::Surface>, tex::SurfaceInfo);

impl<R: Resources> Surface<R> {
    /// Get surface info
    pub fn get_info(&self) -> &tex::SurfaceInfo { &self.1 }
}

/// Texture Handle
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Texture<R: Resources>(Arc<R::Texture>, tex::TextureInfo);

impl<R: Resources> Texture<R> {
    /// Get texture info
    pub fn get_info(&self) -> &tex::TextureInfo { &self.1 }
}

/// Sampler Handle
#[derive(Clone, Debug, PartialEq)]
pub struct Sampler<R: Resources>(Arc<R::Sampler>, tex::SamplerInfo);

impl<R: Resources> Sampler<R> {
    /// Get sampler info
    pub fn get_info(&self) -> &tex::SamplerInfo { &self.1 }
}

/// Fence Handle
#[derive(Clone, Debug, PartialEq)]
pub struct Fence<R: Resources>(Arc<R::Fence>);

/// Stores reference-counted resources used in a command buffer.
/// Seals actual resource names behind the interface, automatically
/// referencing them both by the Factory on resource creation
/// and the Renderer during CommandBuffer population.
#[allow(missing_docs)]
pub struct Manager<R: Resources> {
    buffers:         Vec<Arc<R::Buffer>>,
    array_buffers:   Vec<Arc<R::ArrayBuffer>>,
    shaders:         Vec<Arc<R::Shader>>,
    programs:        Vec<Arc<R::Program>>,
    pipeline_states: Vec<Arc<R::PipelineState>>,
    frame_buffers:   Vec<Arc<R::FrameBuffer>>,
    surfaces:        Vec<Arc<R::Surface>>,
    textures:        Vec<Arc<R::Texture>>,
    samplers:        Vec<Arc<R::Sampler>>,
    fences:          Vec<Arc<R::Fence>>,
}

/// A service trait to be used by the device implementation
#[allow(missing_docs)]
pub trait Producer<R: Resources> {
    fn make_buffer(&mut self, R::Buffer, BufferInfo) -> RawBuffer<R>;
    fn make_array_buffer(&mut self, R::ArrayBuffer) -> ArrayBuffer<R>;
    fn make_shader(&mut self, R::Shader) -> Shader<R>;
    fn make_program(&mut self, R::Program, shade::ProgramInfo) -> Program<R>;
    fn make_pipeline_state(&mut self, R::PipelineState, pso::PipelineInfo,
                           shade::ProgramInfo) -> PipelineState<R>;
    fn make_frame_buffer(&mut self, R::FrameBuffer) -> FrameBuffer<R>;
    fn make_surface(&mut self, R::Surface, tex::SurfaceInfo) -> Surface<R>;
    fn make_texture(&mut self, R::Texture, tex::TextureInfo) -> Texture<R>;
    fn make_sampler(&mut self, R::Sampler, tex::SamplerInfo) -> Sampler<R>;
    fn make_fence(&mut self, name: R::Fence) -> Fence<R>;

    /// Walk through all the handles, keep ones that are reference elsewhere
    /// and call the provided delete function (resource-specific) for others
    fn clean_with<T,
        A: Fn(&mut T, &R::Buffer),
        B: Fn(&mut T, &R::ArrayBuffer),
        C: Fn(&mut T, &R::Shader),
        D: Fn(&mut T, &R::Program),
        E: Fn(&mut T, &R::PipelineState),
        F: Fn(&mut T, &R::FrameBuffer),
        G: Fn(&mut T, &R::Surface),
        H: Fn(&mut T, &R::Texture),
        I: Fn(&mut T, &R::Sampler),
        J: Fn(&mut T, &R::Fence),
    >(&mut self, &mut T, A, B, C, D, E, F, G, H, I, J);
}

impl<R: Resources> Producer<R> for Manager<R> {
    fn make_buffer(&mut self, name: R::Buffer, info: BufferInfo) -> RawBuffer<R> {
        let r = Arc::new(name);
        self.buffers.push(r.clone());
        RawBuffer(r, info)
    }

    fn make_array_buffer(&mut self, name: R::ArrayBuffer) -> ArrayBuffer<R> {
        let r = Arc::new(name);
        self.array_buffers.push(r.clone());
        ArrayBuffer(r)
    }

    fn make_shader(&mut self, name: R::Shader) -> Shader<R> {
        let r = Arc::new(name);
        self.shaders.push(r.clone());
        Shader(r)
    }

    fn make_program(&mut self, name: R::Program, info: shade::ProgramInfo) -> Program<R> {
        let r = Arc::new(name);
        self.programs.push(r.clone());
        Program(r, info)
    }

    fn make_pipeline_state(&mut self, name: R::PipelineState, info: pso::PipelineInfo,
                           prog_info: shade::ProgramInfo) -> PipelineState<R> {
        let r = Arc::new(name);
        self.pipeline_states.push(r.clone());
        PipelineState(r, info, prog_info)
    }

    fn make_frame_buffer(&mut self, name: R::FrameBuffer) -> FrameBuffer<R> {
        let r = Arc::new(name);
        self.frame_buffers.push(r.clone());
        FrameBuffer(r)
    }

    fn make_surface(&mut self, name: R::Surface, info: tex::SurfaceInfo) -> Surface<R> {
        let r = Arc::new(name);
        self.surfaces.push(r.clone());
        Surface(r, info)
    }

    fn make_texture(&mut self, name: R::Texture, info: tex::TextureInfo) -> Texture<R> {
        let r = Arc::new(name);
        self.textures.push(r.clone());
        Texture(r, info)
    }

    fn make_sampler(&mut self, name: R::Sampler, info: tex::SamplerInfo) -> Sampler<R> {
        let r = Arc::new(name);
        self.samplers.push(r.clone());
        Sampler(r, info)
    }

    fn make_fence(&mut self, name: R::Fence) -> Fence<R> {
        let r = Arc::new(name);
        self.fences.push(r.clone());
        Fence(r)
    }

    fn clean_with<T,
        A: Fn(&mut T, &R::Buffer),
        B: Fn(&mut T, &R::ArrayBuffer),
        C: Fn(&mut T, &R::Shader),
        D: Fn(&mut T, &R::Program),
        E: Fn(&mut T, &R::PipelineState),
        F: Fn(&mut T, &R::FrameBuffer),
        G: Fn(&mut T, &R::Surface),
        H: Fn(&mut T, &R::Texture),
        I: Fn(&mut T, &R::Sampler),
        J: Fn(&mut T, &R::Fence),
    >(&mut self, param: &mut T, fa: A, fb: B, fc: C, fd: D, fe: E, ff: F, fg: G, fh: H, fi: I, fj: J) {
        fn clean_vec<X, Param, Fun>(param: &mut Param, vector: &mut Vec<Arc<X>>, fun: Fun)
            where X: Clone, Fun: Fn(&mut Param, &X)
        {
            let mut temp = Vec::new();
            // delete unique resources and make a list of their indices
            for (i, v) in vector.iter_mut().enumerate() {
                if let Some(x) = Arc::get_mut(v) {
                    fun(param, x);
                    temp.push(i);
                }
            }
            // update the resource vector by removing the elements
            // starting from the last one
            for t in temp.iter().rev() {
                vector.swap_remove(*t);
            }
        }
        clean_vec(param, &mut self.buffers,         fa);
        clean_vec(param, &mut self.array_buffers,   fb);
        clean_vec(param, &mut self.shaders,         fc);
        clean_vec(param, &mut self.programs,        fd);
        clean_vec(param, &mut self.pipeline_states, fe);
        clean_vec(param, &mut self.frame_buffers,   ff);
        clean_vec(param, &mut self.surfaces,        fg);
        clean_vec(param, &mut self.textures,        fh);
        clean_vec(param, &mut self.samplers,        fi);
        clean_vec(param, &mut self.fences,          fj);
    }
}

impl<R: Resources> Manager<R> {
    /// Create a new handle manager
    pub fn new() -> Manager<R> {
        Manager {
            buffers: Vec::new(),
            array_buffers: Vec::new(),
            shaders: Vec::new(),
            programs: Vec::new(),
            pipeline_states: Vec::new(),
            frame_buffers: Vec::new(),
            surfaces: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
            fences: Vec::new()
        }
    }
    /// Clear all references
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.array_buffers.clear();
        self.shaders.clear();
        self.programs.clear();
        self.pipeline_states.clear();
        self.frame_buffers.clear();
        self.surfaces.clear();
        self.textures.clear();
        self.samplers.clear();
    }
    /// Extend with all references of another handle manager
    pub fn extend(&mut self, other: &Manager<R>) {
        self.buffers        .extend(other.buffers        .iter().map(|h| h.clone()));
        self.array_buffers  .extend(other.array_buffers  .iter().map(|h| h.clone()));
        self.shaders        .extend(other.shaders        .iter().map(|h| h.clone()));
        self.programs       .extend(other.programs       .iter().map(|h| h.clone()));
        self.pipeline_states.extend(other.pipeline_states.iter().map(|h| h.clone()));
        self.frame_buffers  .extend(other.frame_buffers  .iter().map(|h| h.clone()));
        self.surfaces       .extend(other.surfaces       .iter().map(|h| h.clone()));
        self.textures       .extend(other.textures       .iter().map(|h| h.clone()));
        self.samplers       .extend(other.samplers       .iter().map(|h| h.clone()));
    }
    /// Count the total number of referenced resources
    pub fn count(&self) -> usize {
        self.buffers.len() +
        self.array_buffers.len() +
        self.shaders.len() +
        self.programs.len() +
        self.pipeline_states.len() +
        self.frame_buffers.len() +
        self.surfaces.len() +
        self.textures.len() +
        self.samplers.len()
    }
    /// Reference a buffer
    pub fn ref_buffer(&mut self, handle: &RawBuffer<R>) -> R::Buffer {
        self.buffers.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference am array buffer
    pub fn ref_array_buffer(&mut self, handle: &ArrayBuffer<R>) -> R::ArrayBuffer {
        self.array_buffers.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a shader
    pub fn ref_shader(&mut self, handle: &Shader<R>) -> R::Shader {
        self.shaders.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a program
    pub fn ref_program(&mut self, handle: &Program<R>) -> R::Program {
        self.programs.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a ppipeline state object
    pub fn ref_pipeline_state(&mut self, handle: &PipelineState<R>) -> R::PipelineState {
        self.pipeline_states.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a frame buffer
    pub fn ref_frame_buffer(&mut self, handle: &FrameBuffer<R>) -> R::FrameBuffer {
        self.frame_buffers.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a surface
    pub fn ref_surface(&mut self, handle: &Surface<R>) -> R::Surface {
        self.surfaces.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a texture
    pub fn ref_texture(&mut self, handle: &Texture<R>) -> R::Texture {
        self.textures.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a sampler
    pub fn ref_sampler(&mut self, handle: &Sampler<R>) -> R::Sampler {
        self.samplers.push(handle.0.clone());
        *handle.0.deref()
    }
    /// Reference a fence
    pub fn ref_fence(&mut self, fence: &Fence<R>) -> R::Fence {
        self.fences.push(fence.0.clone());
        *fence.0.deref()
    }
}
