// mpicc compute_mst.c -o compute_mst -lrustc -L./rustc/target/debug
// LD_LIBRARY_PATH=rustc/target/debug/ mpirun --use-hwthread-cpus -np 3 ./compute_mst

#include <stdio.h>
#include <stdlib.h>
#include "mpi.h"

extern void boruvka(int *subgraph, int subgraph_length, int size, int size_global, int shift, int *output, int output_lenght);
const int N = 6;

struct subset {
	int parent;
	int rank;
};
int find(struct subset subsets[], int i) {
	if (subsets[i].parent != i) {
		subsets[i].parent = find(subsets, subsets[i].parent);
	}
	return subsets[i].parent;
}
void Union(struct subset subsets[], int x, int y) {
	int xroot = find(subsets, x);
	int yroot = find(subsets, y);
	if (subsets[xroot].rank<subsets[yroot].rank) {
		subsets[xroot].parent = yroot;
	} else if (subsets[xroot].rank>subsets[yroot].rank) {
		subsets[yroot].parent = xroot;
	}
	else {
		subsets[yroot].parent = xroot;
		subsets[xroot].rank++;
	}
}
void kruskal(int edges[2*N-3][2]) {
	int e = 0;
	int i = 0;
	int MST[N-1][2];
	
	struct subset* subsets = (struct subset*) malloc(N*sizeof(struct subset));
	for (int v = 0; v<N; v++) {
		subsets[v].parent = v;
		subsets[v].rank = 0;
	}
	
	while (e<N-1) {
		int u = edges[i][0];
		int v = edges[i][1];
		int subset_u = find(subsets, u);
		int subset_v = find(subsets, v);
		//subset_u != subset_v => adding this edges doesn't cause cycle
		if (subset_u != subset_v) {
			MST[e][0] = u;
			MST[e][1] = v;
			Union(subsets, subset_u, subset_v);
			e++;
		}
		i++;
	}
	for (int i = 0; i<N-2; i++) {
		printf("%d",MST[i][0]);
		printf("-%d/",MST[i][1]);
	}
	printf("%d",MST[N-2][0]);
	printf("-%d\n",MST[N-2][1]);
}

int main(int argc, char *argv[]) {
	int rank, size;
	MPI_Init(&argc, &argv);
	int s = N*2/3;
	int subgraph[3*s*(s-1)];
	int local_MST[2*(s-1)];
	int *combined_MST = NULL;
	int **graph = NULL;
	MPI_Comm_rank(MPI_COMM_WORLD, &rank);
	MPI_Comm_size(MPI_COMM_WORLD, &size);
	if (rank == 0) {
		combined_MST = (int *)malloc(size*2*(s-1)*sizeof(int));

		graph = (int **)malloc(N*sizeof(int *));
		for (int i=0; i<N; i++)
			graph[i] = (int *)malloc((N-1)*sizeof(int));
		graph[0][0] = 0;graph[0][1] = 4;graph[0][2] = 4;graph[0][3] = 4;graph[0][4] = 1;
		graph[1][0] = 0;graph[1][1] = 2;graph[1][2] = 4;graph[1][3] = 4;graph[1][4] = 4;
		graph[2][0] = 4;graph[2][1] = 2;graph[2][2] = 0;graph[2][3] = 4;graph[2][4] = 4;
		graph[3][0] = 4;graph[3][1] = 4;graph[3][2] = 0;graph[3][3] = 3;graph[3][4] = 4;
		graph[4][0] = 4;graph[4][1] = 4;graph[4][2] = 4;graph[4][3] = 3;graph[4][4] = 0;
		graph[5][0] = 1;graph[5][1] = 4;graph[5][2] = 4;graph[5][3] = 4;graph[5][4] = 0;

		for (int k = 1; k<size; k++) {
			for (int i = 0; i<s; i++) {
				for (int j = 0; j<s-1; j++){
					subgraph[3*(i*(s-1)+j)] = i;
					if (i<=j) {
						subgraph[3*(i*(s-1)+j)+1] = j+1;
					} else {
						subgraph[3*(i*(s-1)+j)+1] = j;
					}
					subgraph[3*(i*(s-1)+j)+2] = graph[(i+N/3*k)%N][(j+N/3*k)%N];
				}
			}
			MPI_Send(subgraph, 3*s*(s-1), MPI_INT, k, 0, MPI_COMM_WORLD);
		}
		for (int i = 0; i<s; i++) {
			for (int j = 0; j<s-1; j++) {
				subgraph[3*(i*(s-1)+j)] = i; 
				if (i<=j) {
					subgraph[3*(i*(s-1)+j)+1] = j+1;
				} else {
					subgraph[3*(i*(s-1)+j)+1] = j;
				}
				subgraph[3*(i*s+j)+2] = graph[i][j];
			}
		}
	}
	else {
		MPI_Recv(subgraph, 3*s*(s-1), MPI_INT, 0, 0, MPI_COMM_WORLD, MPI_STATUS_IGNORE);		
	}
	for (int k = 0; k<size; k++) {
		if (rank == k ) {
			boruvka(subgraph, 3*s*(s-1), s, N, k*N/3, local_MST, 2*(s-1));
		}
	}
	MPI_Gather(local_MST, 2*(s-1), MPI_INT, combined_MST, 2*(s-1), MPI_INT,0, MPI_COMM_WORLD);
	if (rank == 0) {
		int reshaped_combined_MST[3*(s-1)][2];
		for (int i = 0; i<3*(s-1); i++) {
			reshaped_combined_MST[i][0] = combined_MST[2*i];
			reshaped_combined_MST[i][1] = combined_MST[2*i+1];
		}
		int compare (const void *p1, const void *p2) {
			int *arr1 = (int*)p1;
			int *arr2 = (int*)p2;
			return graph[arr1[1]][arr1[0]]-graph[arr2[1]][arr2[0]];
	}
	qsort(reshaped_combined_MST, 3*(s-1), 2*sizeof(int), compare);
	kruskal(reshaped_combined_MST);
	}
	MPI_Finalize();
	return 0;
}
