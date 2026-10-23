use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod twisty_puzzles_py {
    use pyo3::{exceptions::PyValueError, prelude::*};
    use rubiks_cube::{Cube3x3, Puzzle};

    #[pyclass(name = "Cube3x3")]
    struct PyCube3x3(Cube3x3);

    #[pymethods]
    impl PyCube3x3 {
        #[new]
        fn new() -> Self {
            Self(Cube3x3::default())
        }

        fn __repr__(&self) -> String {
            format!("{:?}", self.0)
        }

        fn __str__(&self) -> String {
            self.0.to_string()
        }

        fn apply(&mut self, moves: &str) -> PyResult<()> {
            use std::error::Error;

            fn report(err: &dyn Error) -> String {
                std::iter::successors(Some(err), |&e| e.source())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(": ")
            }

            self.0 = self
                .0
                .apply_moves(moves)
                .map_err(|e| PyValueError::new_err(report(&e)))?;

            Ok(())
        }

        fn is_solved(&self) -> bool {
            self.0.is_solved()
        }
    }
}
