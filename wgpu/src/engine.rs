use crate::graphics::{Antialiasing, Shell};
use crate::primitive;
use crate::quad;
use crate::text;
use crate::triangle;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

/// Serialises queue submissions against surface reconfiguration.
///
/// A `Surface::configure` waits for the GPU to go idle. A submission from
/// another thread during that wait breaks the wait on wgpu and, on Vulkan,
/// violates the external synchronisation `vkDeviceWaitIdle` requires. Every
/// submission must therefore hold the lock that `configure_surface` holds.
pub(crate) fn lock_submits(lock: &Mutex<()>) -> MutexGuard<'_, ()> {
    lock.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Clone)]
pub struct Engine {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) format: wgpu::TextureFormat,

    pub(crate) quad_pipeline: quad::Pipeline,
    pub(crate) text_pipeline: text::Pipeline,
    pub(crate) triangle_pipeline: triangle::Pipeline,
    #[cfg(any(feature = "image", feature = "svg"))]
    pub(crate) image_pipeline: crate::image::Pipeline,
    pub(crate) primitive_storage: Arc<RwLock<primitive::Storage>>,
    submit_lock: Arc<Mutex<()>>,
    _shell: Shell,
}

impl Engine {
    pub fn new(
        _adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        antialiasing: Option<Antialiasing>, // TODO: Initialize AA pipelines lazily
        shell: Shell,
    ) -> Self {
        Self {
            format,

            quad_pipeline: quad::Pipeline::new(&device, format),
            text_pipeline: text::Pipeline::new(&device, &queue, format),
            triangle_pipeline: triangle::Pipeline::new(&device, format, antialiasing),

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline: {
                let backend = _adapter.get_info().backend;

                crate::image::Pipeline::new(&device, format, backend)
            },

            primitive_storage: Arc::new(RwLock::new(primitive::Storage::default())),
            submit_lock: Arc::new(Mutex::new(())),

            device,
            queue,
            _shell: shell,
        }
    }

    /// Holds back every queue submission while it is alive.
    ///
    /// `Compositor::configure_surface` holds this for the duration of the
    /// reconfigure. Any other thread that submits must take it too.
    pub(crate) fn lock_submits(&self) -> MutexGuard<'_, ()> {
        lock_submits(&self.submit_lock)
    }

    /// Submits `command_buffers`, never concurrently with a reconfigure.
    pub(crate) fn submit(
        &self,
        command_buffers: impl IntoIterator<Item = wgpu::CommandBuffer>,
    ) -> wgpu::SubmissionIndex {
        let _guard = self.lock_submits();
        self.queue.submit(command_buffers)
    }

    #[cfg(any(feature = "image", feature = "svg"))]
    pub fn create_image_cache(&self) -> crate::image::Cache {
        self.image_pipeline.create_cache(
            &self.device,
            &self.queue,
            &self._shell,
            Arc::clone(&self.submit_lock),
        )
    }

    pub fn trim(&mut self) {
        self.text_pipeline.trim();

        self.primitive_storage
            .write()
            .expect("primitive storage should be writable")
            .trim();
    }
}
