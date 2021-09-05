extern crate libc;
use hora::core::ann_index::SerializableIndex;
use libc::{c_char, c_float, c_int};
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

trait ANNIndexer:
    hora::core::ann_index::ANNIndex<f32, usize> + hora::core::ann_index::SerializableIndex<f32, usize>
{
}

impl ANNIndexer for hora::index::bruteforce_idx::BruteForceIndex<f32, usize> {}

pub fn metrics_transform(s: &str) -> hora::core::metrics::Metric {
    match s {
        "angular" => hora::core::metrics::Metric::Angular,
        "manhattan" => hora::core::metrics::Metric::Manhattan,
        "dot_product" => hora::core::metrics::Metric::DotProduct,
        "euclidean" => hora::core::metrics::Metric::Euclidean,
        "cosine_similarity" => hora::core::metrics::Metric::CosineSimilarity,
        _ => hora::core::metrics::Metric::Unknown,
    }
}

lazy_static! {
    static ref ANN_INDEX_MANAGER: Mutex<HashMap<String, Box<dyn ANNIndexer>>> =
        Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn index(name: *const c_char, dimension: c_int) {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let idx_dimension = dimension as usize;

    ANN_INDEX_MANAGER.lock().unwrap().insert(
        idx_name,
        Box::new(
            hora::index::bruteforce_idx::BruteForceIndex::<f32, usize>::new(
                idx_dimension,
                &hora::index::bruteforce_params::BruteForceParams::default(),
            ),
        ),
    );
}

#[no_mangle]
pub extern "C" fn add(
    name: *const c_char,
    original_features: *const c_float,
    original_features_size: c_int,
    features_idx: c_int,
) {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let features = unsafe {
        std::slice::from_raw_parts(
            original_features as *const f32,
            original_features_size as usize,
        )
    };
    let idx = features_idx as usize;

    match &mut ANN_INDEX_MANAGER.lock().unwrap().get_mut(&idx_name) {
        Some(index) => {
            let n = hora::core::node::Node::new_with_idx(features, idx);
            index.add_node(&n).unwrap();
        }
        None => {}
    }
}

#[no_mangle]
pub extern "C" fn build(name: *const c_char, mt: *const c_char) {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let metric: String = unsafe { CStr::from_ptr(mt) }.to_str().unwrap().to_string();

    match &mut ANN_INDEX_MANAGER.lock().unwrap().get_mut(&idx_name) {
        Some(index) => {
            index.build(metrics_transform(&metric)).unwrap();
        }
        None => {}
    }
}

#[no_mangle]
pub extern "C" fn search(
    name: *const c_char,
    k: c_int,
    original_features: *const c_float,
    original_features_size: c_int,
) -> *mut usize {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let features = unsafe {
        std::slice::from_raw_parts(
            original_features as *const f32,
            original_features_size as usize,
        )
    };
    let topk = k as usize;
    let mut result: Vec<usize> = Vec::new();

    if let Some(index) = ANN_INDEX_MANAGER.lock().unwrap().get(&idx_name) {
        result = index
            .search(features, topk)
            .iter()
            .map(|x| *x as usize)
            .collect();
    }

    let ptr = result.as_mut_ptr();

    std::mem::forget(result); // so that it is not destructed at the end of the scope

    ptr
}

#[no_mangle]
pub extern "C" fn load(name: *const c_char, _file_path: *const c_char) {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let file_path: String = unsafe { CStr::from_ptr(_file_path) }
        .to_str()
        .unwrap()
        .to_string();
    ANN_INDEX_MANAGER.lock().unwrap().insert(
        idx_name,
        Box::new(
            hora::index::bruteforce_idx::BruteForceIndex::<f32, usize>::load(&file_path).unwrap(),
        ),
    );
}

#[no_mangle]
pub extern "C" fn dump(name: *const c_char, _file_path: *const c_char) {
    let idx_name: String = unsafe { CStr::from_ptr(name) }
        .to_str()
        .unwrap()
        .to_string();
    let file_path: String = unsafe { CStr::from_ptr(_file_path) }
        .to_str()
        .unwrap()
        .to_string();

    if let Some(index) = ANN_INDEX_MANAGER.lock().unwrap().get_mut(&idx_name) {
        index.dump(&file_path).unwrap();
    }
}
