use core::mem::MaybeUninit;

use alloc::rc::Rc;

use crate::driver::sys::{cuMemAllocAsync, cuMemFreeAsync, CUdeviceptr};
use crate::prelude::*;

pub trait RequiresWorkspace {
    fn get_workspace_size(&self) -> CudaCudnnResult<usize>;
    fn execute(
        &mut self,
        workspace_allocation: CUdeviceptr,
        workspace_size: usize,
    ) -> CudaCudnnResult<()>;
}
pub struct WithWorkspace<T> {
    pub(crate) data: T,
    pub(crate) workspace_allocation: CUdeviceptr,
    pub(crate) workspace_size: usize,
    pub(crate) device: Rc<CudaDevice>,
}
impl<T: RequiresWorkspace> WithWorkspace<T> {
    pub fn create(data: T, device: Rc<CudaDevice>) -> CudaCudnnResult<Self> {
        let workspace_size = data.get_workspace_size()?;
        let workspace_allocation = unsafe {
            let mut dev_ptr = MaybeUninit::uninit();
            cuMemAllocAsync(dev_ptr.as_mut_ptr(), workspace_size, device.cu_stream).result()?;
            dev_ptr.assume_init()
        };
        Ok(Self {
            data,
            workspace_size,
            workspace_allocation,
            device,
        })
    }

    pub fn execute(&mut self) -> CudaCudnnResult<()> {
        self.data
            .execute(self.workspace_allocation, self.workspace_size)
    }
}
impl<T> Drop for WithWorkspace<T> {
    fn drop(&mut self) {
        unsafe {
            cuMemFreeAsync(self.workspace_allocation, self.device.cu_stream)
                .result()
                .unwrap()
        }
    }
}