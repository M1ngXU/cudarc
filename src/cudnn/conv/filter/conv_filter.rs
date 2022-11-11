use core::ffi::c_void;

use alloc::rc::Rc;

use super::super::super::sys::*;
use super::descriptor::FilterDescriptor;
use crate::cudarc::CudaUniquePtr;
use crate::prelude::*;

pub struct Filter<T, const C_OUT: usize, const C_IN: usize, const H: usize, const W: usize> {
    descriptor: Rc<FilterDescriptor<T, C_OUT, C_IN, H, W>>,
    data: CudaRc<[[[[T; W]; H]; C_IN]; C_OUT]>,
}
impl<T, const C_OUT: usize, const C_IN: usize, const H: usize, const W: usize> Clone
    for Filter<T, C_OUT, C_IN, H, W>
{
    fn clone(&self) -> Self {
        Self {
            descriptor: Rc::clone(&self.descriptor),
            data: self.data.clone(),
        }
    }
}
impl<T: TensorDataType, const C_OUT: usize, const C_IN: usize, const H: usize, const W: usize>
    Filter<T, C_OUT, C_IN, H, W>
where
    [(); W * H * C_IN * C_OUT]:,
{
    #[inline(always)]
    pub fn get_descriptor(&self) -> cudnnFilterDescriptor_t {
        self.descriptor.get_descriptor()
    }

    #[inline(always)]
    pub fn get_data_ptr(&self) -> *const c_void {
        self.data.t_cuda.cu_device_ptr as *const _
    }

    #[inline(always)]
    pub fn get_data_ptr_mut(&self) -> *mut c_void {
        self.data.t_cuda.cu_device_ptr as *mut _
    }

    #[inline(always)]
    pub fn get_data(&self) -> CudaCudnnResult<Rc<[[[[T; W]; H]; C_IN]; C_OUT]>> {
        self.data.clone().into_host().into_cuda_cudnn_result()
    }

    pub fn create(allocation: CudaRc<[[[[T; W]; H]; C_IN]; C_OUT]>) -> CudaCudnnResult<Self> {
        Ok(Self {
            descriptor: Rc::new(FilterDescriptor::create()?),
            data: allocation,
        })
    }

    pub unsafe fn alloc_uninit(device: &Rc<CudaDevice>) -> CudaCudnnResult<Self> {
        Self::create(CudaRc {
            t_cuda: Rc::new(CudaUniquePtr::alloc(device)?),
            t_host: None,
        })
    }

    pub fn alloc_with(
        device: &Rc<CudaDevice>,
        value: [[[[T; W]; H]; C_IN]; C_OUT],
    ) -> CudaCudnnResult<Self> {
        Self::create(device.take(Rc::new(value))?)
    }
}