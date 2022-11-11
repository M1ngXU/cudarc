use core::mem::MaybeUninit;

use alloc::rc::Rc;

use super::super::sys::*;
use crate::prelude::*;

/// A struct that holds all the data to calculate `dx` by `y`, the filter and
/// `dy`.
pub struct Convolution2DBackward<
    T,
    const H: usize,
    const W: usize,
    const P_H: usize,
    const P_W: usize,
    const V_S: usize,
    const H_S: usize,
    const N: usize,
    const C_IN: usize,
    const C_OUT: usize,
    const F_H: usize,
    const F_W: usize,
    const S_H: usize,
    const S_W: usize,
> where
    [(); ConvolutionOutput::<H, P_H, F_H, S_H>::SIZE]:,
    [(); ConvolutionOutput::<W, P_W, F_W, S_W>::SIZE]:,
{
    descriptor: Rc<ConvolutionDescriptor>,
    dy: Tensor4D<
        T,
        N,
        C_OUT,
        { ConvolutionOutput::<H, P_H, F_H, S_H>::SIZE },
        { ConvolutionOutput::<W, P_W, F_W, S_W>::SIZE },
    >,
    filter: Filter<T, C_OUT, C_IN, F_H, F_W>,
    dx: Tensor4D<T, N, C_IN, H, W>,
    cudnn_handle: Rc<CudnnHandle>,
}
impl<
        T: TensorDataType,
        const H: usize,
        const W: usize,
        const P_H: usize,
        const P_W: usize,
        const V_S: usize,
        const H_S: usize,
        const N: usize,
        const C_IN: usize,
        const C_OUT: usize,
        const F_H: usize,
        const F_W: usize,
        const S_H: usize,
        const S_W: usize,
    > Convolution2DBackward<T, H, W, P_H, P_W, V_S, H_S, N, C_IN, C_OUT, F_H, F_W, S_H, S_W>
where
    [(); ConvolutionOutput::<H, P_H, F_H, S_H>::SIZE]:,
    [(); ConvolutionOutput::<W, P_W, F_W, S_W>::SIZE]:,
{
    pub fn create(
        cudnn_handle: Rc<CudnnHandle>,
        descriptor: Rc<ConvolutionDescriptor>,
        dy: Tensor4D<
            T,
            N,
            C_OUT,
            { ConvolutionOutput::<H, P_H, F_H, S_H>::SIZE },
            { ConvolutionOutput::<W, P_W, F_W, S_W>::SIZE },
        >,
        filter: Filter<T, C_OUT, C_IN, F_H, F_W>,
        dx: Tensor4D<T, N, C_IN, H, W>,
    ) -> Self {
        Self {
            cudnn_handle,
            descriptor,
            dy,
            filter,
            dx,
        }
    }
}
impl<
        T: TensorDataType,
        const H: usize,
        const W: usize,
        const P_H: usize,
        const P_W: usize,
        const V_S: usize,
        const H_S: usize,
        const N: usize,
        const C_IN: usize,
        const C_OUT: usize,
        const F_H: usize,
        const F_W: usize,
        const S_H: usize,
        const S_W: usize,
    > RequiresAlgorithmWithWorkspace<cudnnConvolutionBwdDataAlgoPerf_t>
    for Convolution2DBackward<T, H, W, P_H, P_W, V_S, H_S, N, C_IN, C_OUT, F_H, F_W, S_H, S_W>
where
    [(); ConvolutionOutput::<H, P_H, F_H, S_H>::SIZE]:,
    [(); ConvolutionOutput::<W, P_W, F_W, S_W>::SIZE]:,
    [(); F_W * F_H * C_IN * C_OUT]:,
{
    fn get_workspace_size(
        &self,
        algorithm: &cudnnConvolutionBwdDataAlgoPerf_t,
    ) -> CudaCudnnResult<usize> {
        let mut workspace_size = MaybeUninit::uninit();
        unsafe {
            cudnnGetConvolutionBackwardDataWorkspaceSize(
                self.cudnn_handle.get_handle(),
                self.filter.get_descriptor(),
                self.dy.get_descriptor(),
                self.descriptor.0,
                self.dx.get_descriptor(),
                algorithm.algo,
                workspace_size.as_mut_ptr(),
            )
            .result()?;
            Ok(workspace_size.assume_init())
        }
    }

    fn get_algorithm(&self) -> CudaCudnnResult<cudnnConvolutionBwdDataAlgoPerf_t> {
        let mut output_amount = MaybeUninit::uninit();
        let mut algorithm = MaybeUninit::uninit();
        unsafe {
            cudnnGetConvolutionBackwardDataAlgorithm_v7(
                self.cudnn_handle.get_handle(),
                self.filter.get_descriptor(),
                self.dy.get_descriptor(),
                self.descriptor.0,
                self.dx.get_descriptor(),
                1,
                output_amount.as_mut_ptr(),
                algorithm.as_mut_ptr(),
            )
            .result()?;
            assert_eq!(
                output_amount.assume_init(),
                1,
                "cudnnGetConvolutionBackwardDataAlgorithm_v7 returned 0 algorithms"
            );
            Ok(algorithm.assume_init())
        }
    }

    fn execute(
        &mut self,
        algorithm: &cudnnConvolutionBwdDataAlgoPerf_t,
        workspace_allocation: crate::driver::sys::CUdeviceptr,
        workspace_size: usize,
    ) -> CudaCudnnResult<()> {
        unsafe {
            cudnnConvolutionBackwardData(
                self.cudnn_handle.get_handle(),
                &T::ONE as *const _ as *const _,
                self.filter.get_descriptor(),
                self.filter.get_data_ptr(),
                self.dy.get_descriptor(),
                self.dy.get_data_ptr(),
                self.descriptor.0,
                algorithm.algo,
                workspace_allocation as *mut _,
                workspace_size,
                &T::ZERO as *const _ as *const _,
                self.dx.get_descriptor(),
                self.dx.get_data_ptr_mut(),
            )
        }
        .result()
    }
}