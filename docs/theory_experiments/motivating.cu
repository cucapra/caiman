#include <stdio.h>

__global__ void square(int* array, int n)
{
	int tid = blockDim.x * blockIdx.x + threadIdx.x;
	if (tid >= n)
		return;
	array[tid] = array[tid] * array[tid];
}

__host__ void caller()
{
	int* array = nullptr;
	cudaMalloc(& array, sizeof(int));
	int element = 5;
	printf("element: %i\n", element);
	cudaMemcpy(array, & element, sizeof(int), cudaMemcpyHostToDevice);
	square<<<1, 1>>>(array, 1);
	cudaMemcpy(& element, array, sizeof(int), cudaMemcpyDeviceToHost);
	cudaFree(array);
	printf("element: %i\n", element);
}

__device__ __host__ void helper()
{
	
}

//https://developer.nvidia.com/blog/unified-memory-cuda-beginners/
//https://leimao.github.io/blog/CUDA-Stream/
//https://docs.nvidia.com/cuda/cuda-c-programming-guide/index.html
//https://stackoverflow.com/questions/21986542/is-cudamallocmanaged-slower-than-cudamalloc
//https://stackoverflow.com/questions/39782746/why-is-nvidia-pascal-gpus-slow-on-running-cuda-kernels-when-using-cudamallocmana/40011988#40011988
//https://stackoverflow.com/questions/65501537/cudamallocmanaged-unified-memory-with-cublas
//https://developer.nvidia.com/blog/how-overlap-data-transfers-cuda-cc/
//https://developer.nvidia.com/blog/unified-memory-cuda-beginners/
//https://stackoverflow.com/questions/8473617/are-cuda-kernel-calls-synchronous-or-asynchronous
__global__ void task_a(int* data, int n)
{
	int tid = blockDim.x * blockIdx.x + threadIdx.x;
	if (tid >= n)
		return;
	data[tid] += 1;
}

__host__ void task_b(int* data, int n)
{
	for (int tid = 0; tid < n; tid++)
		data[tid] += 1;
}

__host__ void task_c(int* data_a, const int* data_b, int n)
{
	for (int tid = 0; tid < n; tid++)
		data_a[tid] += data_b[tid];
}


__global__ void task_b_device(int* data, int n)
{
	int tid = blockDim.x * blockIdx.x + threadIdx.x;
	if (tid >= n)
		return;
	data[tid] += 1;
}

__global__ void task_c_device(int* data_a, const int* data_b, int n)
{
	int tid = blockDim.x * blockIdx.x + threadIdx.x;
	if (tid >= n)
		return;
	data_a[tid] += data_b[tid];
}

__global__ void task_d(int* data, int n)
{
	int tid = blockDim.x * blockIdx.x + threadIdx.x;
	if (tid >= n)
		return;
	data[tid] += 1;
}

__host__ void schedule_a(int input, int* output)
{
	int* cpu_data_a = (int*) malloc(sizeof(int));
	int* cpu_data_b = (int*) malloc(sizeof(int));
	int* gpu_data_a = NULL;
	//int* gpu_data_b = NULL;
	cudaMalloc(& gpu_data_a, sizeof(int));
	//cudaMalloc(& gpu_data_b, sizeof(int));

	memcpy(cpu_data_a, & input, sizeof(int));
	memcpy(cpu_data_b, & input, sizeof(int));
	cudaMemcpy(gpu_data_a, & input, sizeof(int), cudaMemcpyHostToDevice);
	//cudaMemcpy(gpu_data_b, & input, sizeof(int), cudaMemcpyHostToDevice);

	printf("Line %i: %i\n", __LINE__, input);
	printf("Line %i: %i\n", __LINE__, * cpu_data_a);
	printf("Line %i: %i\n", __LINE__, * cpu_data_b);

	task_a<<<1, 1>>>(gpu_data_a, 1);
	task_b(cpu_data_b, 1);
	cudaMemcpy(cpu_data_a, gpu_data_a, sizeof(int), cudaMemcpyDeviceToHost);
	printf("Line %i: %i\n", __LINE__, * cpu_data_a);
	printf("Line %i: %i\n", __LINE__, * cpu_data_b);
	task_c(cpu_data_a, cpu_data_b, 1);
	printf("Line %i: %i\n", __LINE__, * cpu_data_a);
	printf("Line %i: %i\n", __LINE__, * cpu_data_b);
	cudaMemcpy(gpu_data_a, cpu_data_a, sizeof(int), cudaMemcpyHostToDevice);
	task_d<<<1, 1>>>(gpu_data_a, 1);

	cudaMemcpy(output, gpu_data_a, sizeof(int), cudaMemcpyDeviceToHost);

	free(cpu_data_a);
	free(cpu_data_b);
	cudaFree(gpu_data_a);
	//cudaFree(gpu_data_b);
}

__host__ void schedule_a2(int input, int* output)
{
	int* data_a = (int*) malloc(sizeof(int));
	int* data_b = (int*) malloc(sizeof(int));
	cudaMallocManaged(& data_a, sizeof(int));
	cudaMallocManaged(& data_b, sizeof(int));

	memcpy(data_a, & input, sizeof(int));
	memcpy(data_b, & input, sizeof(int));

	printf("Line %i: %i\n", __LINE__, input);
	printf("Line %i: %i\n", __LINE__, * data_a);
	printf("Line %i: %i\n", __LINE__, * data_b);

	task_a<<<1, 1>>>(data_a, 1);
	task_b(data_b, 1);
	cudaStreamSynchronize(0);
	printf("Line %i: %i\n", __LINE__, * data_a);
	printf("Line %i: %i\n", __LINE__, * data_b);
	task_c(data_a, data_b, 1);
	printf("Line %i: %i\n", __LINE__, * data_a);
	printf("Line %i: %i\n", __LINE__, * data_b);
	task_d<<<1, 1>>>(data_a, 1);
	cudaStreamSynchronize(0);

	memcpy(output, data_a, sizeof(int));

	cudaFree(data_a);
	cudaFree(data_b);
}

__host__ void schedule_b(int input, int* output)
{
	int* cpu_data_a = (int*) malloc(sizeof(int));
	int* cpu_data_b = (int*) malloc(sizeof(int));
	int* gpu_data_a = NULL;
	//int* gpu_data_b = NULL;
	cudaMalloc(& gpu_data_a, sizeof(int));
	//cudaMalloc(& gpu_data_b, sizeof(int));

	memcpy(cpu_data_a, & input, sizeof(int));
	memcpy(cpu_data_b, & input, sizeof(int));
	cudaMemcpyAsync(gpu_data_a, & input, sizeof(int), cudaMemcpyHostToDevice, 0);
	//cudaMemcpy(gpu_data_b, & input, sizeof(int), cudaMemcpyHostToDevice);

	task_a<<<1, 1, 0, 0>>>(gpu_data_a, 1);
	cudaMemcpyAsync(cpu_data_a, gpu_data_a, sizeof(int), cudaMemcpyDeviceToHost, 0);
	task_b(cpu_data_b, 1);
	cudaStreamSynchronize(0);
	task_c(cpu_data_a, cpu_data_b, 1);
	cudaMemcpyAsync(gpu_data_a, cpu_data_a, sizeof(int), cudaMemcpyHostToDevice, 0);
	task_d<<<1, 1>>>(gpu_data_a, 1);

	cudaMemcpy(output, gpu_data_a, sizeof(int), cudaMemcpyDeviceToHost);

	free(cpu_data_a);
	free(cpu_data_b);
	cudaFree(gpu_data_a);
	//cudaFree(gpu_data_b);
}

__host__ void schedule_c(int input, int* output)
{
	int* gpu_data_a = NULL;
	int* gpu_data_b = NULL;
	cudaMalloc(& gpu_data_a, sizeof(int));
	cudaMalloc(& gpu_data_b, sizeof(int));

	cudaMemcpyAsync(gpu_data_a, & input, sizeof(int), cudaMemcpyHostToDevice, 0);
	cudaMemcpyAsync(gpu_data_b, & input, sizeof(int), cudaMemcpyHostToDevice, 0);

	task_a<<<1, 1, 0, 0>>>(gpu_data_a, 1);
	task_b_device<<<1, 1, 0, 0>>>(gpu_data_b, 1);
	task_c_device<<<1, 1, 0, 0>>>(gpu_data_a, gpu_data_b, 1);
	task_d<<<1, 1, 0, 0>>>(gpu_data_a, 1);

	cudaMemcpy(output, gpu_data_a, sizeof(int), cudaMemcpyDeviceToHost);

	cudaFree(gpu_data_a);
	cudaFree(gpu_data_b);
}

int main()
{
	caller();
	int input = 3, output = 0;
	schedule_a(input, & output);
	printf("schedule_a: %i in %i out\n", input, output);
	output = 0;
	schedule_a2(input, & output);
	printf("schedule_a2: %i in %i out\n", input, output);
	output = 0;
	schedule_b(input, & output);
	printf("schedule_b: %i in %i out\n", input, output);
	output = 0;
	schedule_c(input, & output);
	printf("schedule_c: %i in %i out\n", input, output);
	return 0;
}