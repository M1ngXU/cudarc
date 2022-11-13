#!/bin/bash

function print_done() {
    echo -e "\x1b[32mDone!\x1b[m"
}

BASEDIR=$(dirname "$0")
KERNEL_FUNCTION_START="extern \"C\" __global__ void"
cd "$BASEDIR" || return

echo "Generating custom_kernels.cu file ..."
echo "/* automatically generated by compile_custom_kernels.sh */" > custom_kernels.cu
for f in ./custom_kernels/**.cu; do
    echo -en "\tCopying $(basename "$f") ..."
    cat "$f" >> custom_kernels.cu
    print_done
done

echo >> custom_kernels.cu

echo -n "Duplicating 'f32' functions with 'float's to 'f64' and 'double's ..."
sed -E "s/^$KERNEL_FUNCTION_START ([a-zA-Z0-9_]+)_f32(.*)$/$KERNEL_FUNCTION_START \1_f64\2/g" custom_kernels/f_kernels.cu | sed 's/float/double/g' >>custom_kernels.cu
print_done

echo -n "Removing new lines and comments ..."
sed -i '/^$/d' custom_kernels.cu
sed -i '/^\/\/.*$/d' custom_kernels.cu
print_done

echo -n "Creating artifacts ..."
mkdir -p artifacts
cd artifacts || return
print_done

echo -n "Generating custom_kernels.ptx file with nvcc ..."
nvcc -keep ../custom_kernels.cu > /dev/null 2> /dev/null
print_done

echo -n "Moving custom_kernels.ptx ..."
mv custom_kernels.ptx ..
print_done

cd ..

echo -n "Reading kernel function names ..."
(sed -n "/^$KERNEL_FUNCTION_START/p" custom_kernels.cu | sed -E "s/^$KERNEL_FUNCTION_START ([a-zA-Z0-9_]+).*$/\t\\\"\1\\\",/g") | tee >(wc -l > .tmp_lines) > .tmp_data
print_done

echo -n "Creating custom_kernel_functions_names.rs ..."
(
    echo -e '/* automatically generated by compile_custom_kernels.sh */\npub const FUNCTION_NAMES: [&str; ';
    head -1 .tmp_lines; echo '] = [';
    cat .tmp_data; echo -e '];'
) > custom_kernel_functions_names.rs
print_done


echo -n "Removing artifacts/temp files ..."
rm .tmp_data
rm .tmp_lines
rm -rf artifacts
print_done

echo -n "Formatting custom_kernel_functions_names.rs ..."
rustfmt custom_kernel_functions_names.rs
echo -en "\x1b[1m"
print_done