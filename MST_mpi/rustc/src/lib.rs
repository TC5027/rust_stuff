use std::slice;
use rayon::prelude::*;
use std::collections::{HashMap};
use std::cmp::{min,max};

/// boruvka algorithm, designed for complete graph
#[no_mangle]
pub extern "C" fn boruvka(c_array: *mut i32, length: usize, size: usize, size_global: usize, shift: usize, output: *mut i32, output_length: usize) {
    let edges: &mut [i32] = unsafe{slice::from_raw_parts_mut(c_array, length as usize)};
    let mut edges = edges.par_chunks(3)
            .map(|v| (v[0] as usize, v[1] as usize, v[2]))
            .collect::<Vec<(usize,usize,i32)>>();
    // where we store the edges of MST, one edge = a pair of vertices
    let mut tree = HashMap::new();
    // number of composants in the current state of the graph
    let mut composants = size;
    // keeps track to which components each vertex belongs
    let mut belongs_to = HashMap::new();
    // initially, one vertex = one component
    for k in 0..size {
        belongs_to.insert(k,k);
    }
    while composants > 1 {
        // STEP search for the lightest edges between distinct components
        let found = edges.par_chunks(composants-1).map(|chunk| {
            let mut mini = std::i32::MAX;
            let mut index = 0;
            for (k,(_,_,w_ij)) in chunk.iter().enumerate() {
                if mini > *w_ij {
                    mini = *w_ij;
                    index = k;
                }
            }
            chunk[index]
        }).collect::<Vec<_>>();
        
        // insert the found edges of the MST
        found.iter().for_each(|&(i,j,_)| {
            tree.insert((min(i,j),max(i,j)), 1);
        });

        // STEP assign subgraph to vertices
        let mut pred = found.par_iter()
                .map(|(i,j,_)| (belongs_to[i],belongs_to[j]))
                .collect::<HashMap<_,_>>();
        pred = (0..composants).into_par_iter().map(|c| {
            let w = pred[&c];
            if pred[&w]==c && c<w {
                (c,c)
            } else {
                (c,w)
            }
        }).collect();
        while (0..composants).into_par_iter().any(|c| pred[&c]!=pred[&pred[&c]]) {
            pred = (0..composants).into_par_iter().map(|c| {
                (c,pred[&pred[&c]])
            }).collect();
        }
        // like that all components of same new component have same values in pred

        // we update the number of remaining components
        composants = size-tree.len();

        // remove the edges within the same component
        let not_within_same = edges.into_par_iter()
                .filter(|(i,j,_)| pred[&belongs_to[i]]!=pred[&belongs_to[j]])
                .collect::<Vec<_>>();

        // concordance helps us mapping the index of components to the range 0..composants
        let mut concordance = pred.values()
                .map(|v| (v.clone(),1 as usize))
                .collect::<HashMap<_,_>>();
        concordance = concordance.keys().enumerate().map(|(i,&j)| (j,i)).collect::<HashMap<_,_>>();
        
        // organize the edges to preserve order so that next iteration 
        // we can re-use the "par_chunks(composants-1)" for found
        let mut organize_by_component : Vec<Vec<(usize, usize, i32)>> = vec![Vec::with_capacity(not_within_same.len()/composants);composants];
        for (i, j, w_ij) in not_within_same {
            organize_by_component[concordance[&pred[&i]]].push((i,j,w_ij));
        }
        
        // we remove the multi-edges between the different components
        // meaning we don't want to have more than 1 edge between 2 distinct components
        edges = (0..composants).into_par_iter().map(|k| {
            let mut minis = vec![(0,0,std::i32::MAX);composants];
            for (i,j,w_ij) in organize_by_component[k].clone() {
                let composant = concordance[&pred[&j]];
                if minis[composant].2 > w_ij {
                    minis[composant] = (i,j,w_ij);
                }
            }
            minis.into_par_iter()
        }).flatten().filter(|(i,j,_)| i!=j).collect();
        
        // we keep the data concerning the component's attributions for each vertex in belongs_to
        for k in 0..size {
            belongs_to.insert(k,concordance[&pred[&belongs_to[&k]]]);
        }
    }
    let output: &mut [i32] = unsafe{slice::from_raw_parts_mut(output, output_length)};
    for (k,(i,j)) in tree.keys().enumerate() {
        output[2*k] = ((i+shift)%size_global) as i32;
        output[2*k+1] = ((j+shift)%size_global) as i32;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
