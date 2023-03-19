use crate::object::{SimulationHandler, RobotHandler};
use super::sampler::{Sampler};

pub struct GraphBasedPlanner
{
    sampler: Box<dyn Sampler>,
    sim_handler: &'static SimulationHandler,
    robot_handler: &'static RobotHandler,
    graph: f32,
}

impl GraphBasedPlanner
{
    pub fn GraphBasedPlanner(sampler: Box<dyn Sampler>, sim_handler: &'static SimulationHandler, robot_handler: &'static RobotHandler) -> GraphBasedPlanner
    {
        GraphBasedPlanner
        {
            sampler,
            sim_handler,
            robot_handler,
            graph: 2.0,
        }
    }

    /* This builds a graph in the whole network using rejection sampling of the environment */
    pub fn build_graph(&mut self)
    {

    } 

    /* Plans a path from start to goal */
    pub fn plan(&self, start: (f32, f32, f32), end: (f32, f32, f32)) -> Vec<(f32, f32, f32)>
    {

        Vec::new()
    }
}