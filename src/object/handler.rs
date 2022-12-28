use super::robot::Robot;
use super::wall::Wall;
use crate::behaviour::traits::{Collidable, Drawable};
use crate::parameter::*;

pub struct RobotHandler
{
    id: usize,
}

pub struct SimulationHandler
{
    robots: Vec<Robot>,
    objects: Vec<Box<dyn Collidable>>,
    artists: Vec<Box<dyn Drawable>>,

}

impl SimulationHandler
{
    pub fn new() -> SimulationHandler
    {
        return SimulationHandler {robots: Vec::new(), objects: Vec::new(), artists: Vec::new(),};
    }

    pub fn add_robot(&mut self, robot: Robot) -> RobotHandler
    {
        self.robots.push(robot);

        return RobotHandler { id: self.robots.len()-1 }
    }

    pub fn add_wall(&mut self, wall: Wall)
    {
        self.objects.push(Box::new(wall.clone()));
        self.artists.push(Box::new(wall.clone()));
    }

    pub fn control(&mut self, robot: &RobotHandler, control: (f32, f32))
    {
        self.robots[robot.id].control(control);
    }

    pub fn step(&mut self)
    {
        // For each robot, perform collision check and then step
        for robot in self.robots.iter_mut()
        {
            let next_pose = robot.next();
            let mut collision: bool = false;
            for object in self.objects.iter()
            {
                
               collision = robot.collision_check_at(object.as_ref(), &next_pose);
               if collision
               {
                break;
               }
            }

            if !collision
            {
                robot.step(&next_pose); 
            }   
        }
    }
    

    pub fn draw(&self)
    {
        for robot in self.robots.iter()
        {
            robot.draw(Self::tf_function);
        }

        for artist in self.artists.iter()
        {
            artist.draw(Self::tf_function);
        }
    }

    fn tf_function(pos: (f32, f32))->(f32, f32)
    {
        let i = (pos.0 - XLIMS.0)/RESOLUTION;
        let j = (YLIMS.1-pos.1)/RESOLUTION;
    
        return (i, j);
    }

}