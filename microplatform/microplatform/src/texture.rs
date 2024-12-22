use crate::{BackendRef, BackendWeakRef, Result};
use crate::types::{TextureData, TextureId};
use alloc::rc::Rc;
use alloc::rc::Weak;

#[derive(Copy, Clone, Debug)]
pub enum TextureKind {
    Static,
    Target,
}

// TODO: add texture formats
pub struct Texture {
    pub(crate) id: TextureId,
    backend: BackendWeakRef,
    kind: TextureKind,
    width: u32,
    height: u32,
}

impl Texture {
    pub(crate) fn new_static(backend: &BackendRef, path: &str) -> Result<Self> {
        let TextureData { id, width, height } = backend.borrow_mut().texture_load(path)?;
        Ok(Self {
            id,
            kind: TextureKind::Static,
            width,
            height,
            backend: Rc::downgrade(backend),
        })
    }

    pub(crate) fn new_target(backend: &BackendRef, w: u32, h: u32) -> Result<Self> {
        let TextureData { id, width, height } = backend.borrow_mut().texture_create(w, h)?;
        Ok(Self {
            id,
            kind: TextureKind::Target,
            width,
            height,
            backend: Rc::downgrade(backend),
        })
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn kind(&self) -> TextureKind {
        self.kind
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if let Some(backend) = Weak::upgrade(&self.backend) {
            let _ = backend.borrow_mut().texture_destroy(self.id);
        }
    }
}
