use crate::env::RustezeEnv;
use rayon::prelude::*;
use std::sync::Mutex;

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
#[cfg(feature = "extension-module")]
use numpy::{PyArray, IntoPyArray, PyArrayMethods};

/// Multi-environment wrapper for parallel execution of multiple Rusteze environments.
/// 
/// This struct holds multiple `RustezeEnv` instances and allows batched operations
/// using Rust's `rayon` library for parallel execution.
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct MultiRustezeEnv {
    envs: Vec<Mutex<RustezeEnv>>,
}

impl MultiRustezeEnv {
    /// Create a new multi-environment with the specified number of environments.
    /// 
    /// # Arguments
    /// * `num_envs` - Number of parallel environments to create.
    /// * `seed` - Base seed for world generation. Each environment gets seed + index.
    /// 
    /// # Returns
    /// A new `MultiRustezeEnv` instance.
    pub fn new(num_envs: usize, seed: u64) -> Self {
        let envs = (0..num_envs)
            .map(|i| Mutex::new(RustezeEnv::new(seed + i as u64)))
            .collect();
        
        Self { envs }
    }
    
    /// Reset all environments and return their observations.
    /// 
    /// # Returns
    /// A vector of observation pixel data (RGB format).
    pub fn reset_all(&mut self) -> Vec<Vec<u8>> {
        self.envs
            .par_iter()
            .map(|env| {
                let mut env = env.lock().unwrap();
                env.reset_internal()
            })
            .collect()
    }
    
    /// Step all environments with the given actions in parallel.
    /// 
    /// # Arguments
    /// * `actions` - Vector of actions, one per environment.
    /// 
    /// # Returns
    /// A tuple containing:
    /// - `observations`: Vector of RGB pixel data.
    /// - `rewards`: Vector of reward values.
    /// - `dones`: Vector of done flags.
    pub fn step_all(&mut self, actions: Vec<crate::game::actions::Action>) -> (Vec<Vec<u8>>, Vec<f32>, Vec<bool>) {
        assert_eq!(actions.len(), self.envs.len(), "Number of actions must match number of environments");
        
        let results: Vec<(Vec<u8>, f32, bool)> = (0..self.envs.len())
            .into_par_iter()
            .map(|i| {
                let mut env = self.envs[i].lock().unwrap();
                env.step_internal(actions[i].clone())
            })
            .collect();
        
        let observations: Vec<Vec<u8>> = results.iter().map(|(obs, _, _)| obs.clone()).collect();
        let rewards: Vec<f32> = results.iter().map(|(_, reward, _)| *reward).collect();
        let dones: Vec<bool> = results.iter().map(|(_, _, done)| *done).collect();
        
        (observations, rewards, dones)
    }
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl MultiRustezeEnv {
    #[new]
    fn py_new(num_envs: usize, seed: u64) -> Self {
        Self::new(num_envs, seed)
    }
    
    fn reset_all(&mut self, py: Python) -> Vec<Py<PyArray<u8, numpy::Ix3>>> {
        let observations = self.reset_all();
        observations
            .into_iter()
            .map(|obs| {
                let arr = PyArray::from_vec_bound(py, obs).reshape([360, 640, 3]).unwrap();
                arr.into()
            })
            .collect()
    }
    
    fn step_all(&mut self, actions: Vec<PyObject>, py: Python) -> PyResult<(Vec<Py<PyArray<u8, numpy::Ix3>>>, Vec<f32>, Vec<bool>)> {
        // Parse actions from Python objects
        let mut rust_actions = Vec::new();
        for action_obj in actions {
            let action: crate::game::actions::Action = if action_obj.is_none(py) {
                crate::game::actions::Action::Noop {}
            } else {
                // Try to parse as JSON string first
                if let Ok(json_str) = action_obj.extract::<String>(py) {
                    crate::game::actions::Action::from_str(&json_str)
                } else {
                    // Try to parse as dict with PlayerInput fields
                    if let Ok(dict) = action_obj.downcast::<pyo3::types::PyDict>(py) {
                        let mut input = crate::game::actions::PlayerInput::default();
                        
                        // Parse camera
                        if let Ok(camera) = dict.get_item("camera") {
                            if let Ok(camera_list) = camera.and_then(|c| c.downcast::<pyo3::types::PyList>()) {
                                if camera_list.len() == 2 {
                                    if let (Ok(h), Ok(v)) = (
                                        camera_list.get_item(0).and_then(|x| x.extract::<f32>()),
                                        camera_list.get_item(1).and_then(|x| x.extract::<f32>()),
                                    ) {
                                        input.camera = Some([h, v]);
                                    }
                                }
                            }
                        }
                        
                        // Parse movement keys
                        if let Ok(val) = dict.get_item("forward").and_then(|x| x.extract::<bool>()) {
                            input.forward = val;
                        }
                        if let Ok(val) = dict.get_item("back").and_then(|x| x.extract::<bool>()) {
                            input.back = val;
                        }
                        if let Ok(val) = dict.get_item("left").and_then(|x| x.extract::<bool>()) {
                            input.left = val;
                        }
                        if let Ok(val) = dict.get_item("right").and_then(|x| x.extract::<bool>()) {
                            input.right = val;
                        }
                        if let Ok(val) = dict.get_item("jump").and_then(|x| x.extract::<bool>()) {
                            input.jump = val;
                        }
                        if let Ok(val) = dict.get_item("attack").and_then(|x| x.extract::<bool>()) {
                            input.attack = val;
                        }
                        
                        crate::game::actions::Action::from_player_input(input)
                    } else {
                        crate::game::actions::Action::Noop {}
                    }
                }
            };
            rust_actions.push(action);
        }
        
        let (observations, rewards, dones) = self.step_all(rust_actions);
        
        let py_observations: Vec<Py<PyArray<u8, numpy::Ix3>>> = observations
            .into_iter()
            .map(|obs| {
                let arr = PyArray::from_vec_bound(py, obs).reshape([360, 640, 3]).unwrap();
                arr.into()
            })
            .collect();
        
        Ok((py_observations, rewards, dones))
    }
    
    fn num_envs(&self) -> usize {
        self.envs.len()
    }
}

