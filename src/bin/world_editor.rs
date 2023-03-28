extern crate macroquad;
extern crate xiron;

use macroquad::prelude::*;
use serde_yaml;

use xiron::parameter::*;
use xiron::parser::*;


fn tf_function(pos: (f32, f32)) -> (f32, f32) {

    let x = pos.0 * RESOLUTION + XLIMS.0;
    let y = YLIMS.1 - pos.1 * RESOLUTION ;

    return (x, y);
}

fn window_conf() -> Conf
{
    Conf
    {
        window_title: "World Editor".to_owned(),
        window_height: ((YLIMS.1 - YLIMS.0) / RESOLUTION) as i32,
        window_resizable: false,
        window_width: ((XLIMS.1 - XLIMS.0) / RESOLUTION) as i32,
        ..Default::default()
    }
}

#[derive(Debug)]
enum Mode
{
    WallStart,
    WallMid((f32, f32, Option<(usize, usize)>)),

    StaticObjStart,
    StaticObjMid((f32, f32)),

    Idle,

}

#[macroquad::main(window_conf)]
async fn main()
{
    let mut current_mode = Mode::Idle;
    let mut walls: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut static_obj: Vec<(f32, f32, f32, f32)> = Vec::new();
    let mut robots: Vec<(f32, f32)> = Vec::new();
    let snap_dist = 5.0;
    
    loop {
        clear_background(WHITE);
        draw_text("W for wall", 25.0, 25.0, 20.0, GREEN);
        draw_text("O for object", 25.0, 50.0, 20.0, GREEN);
        draw_text("Right Click for robot", 25.0, 75.0, 20.0, GREEN);
        draw_text("S for saving the config", 350.0, 25.0, 20.0, BLACK);

        match current_mode
        {
            Mode::Idle => {
                // Here you search for the keys and shift to a mode
                // If L key is down
                draw_text("Current Mode: Idle", 25.0, 100.0, 20.0, RED);
                if is_key_pressed(KeyCode::W)
                {
                    current_mode = Mode::WallStart;
                }

                else if is_key_pressed(KeyCode::O)
                {
                    current_mode = Mode::StaticObjStart;
                }
            },
            Mode::WallStart => {
                // Wait for mouse click to record the point
                draw_text("Current Mode: WallStart", 25.0, 100.0, 20.0, RED);
                if is_mouse_button_pressed(MouseButton::Left)
                {
                    let (px, py) = mouse_position();
                    // If within any distance to any end points, snap to it
                    let mut nearest: Option<(f32, f32)> = None;
                    let mut nearest_dist: f32 = 10000.0;
                    
                    let mut index_in_walls: Option<usize> = None;
                    let mut index_in_list: Option<usize> = None;

                    for i in 0..walls.len()
                    {
                        let line = &walls[i];

                        for j in 0..line.len()
                        {
                            let pt = line[j];
                            let dist = ((pt.0 - px).powf(2.0) + (pt.1 - py).powf(2.0)).sqrt();

                            if dist < snap_dist && dist < nearest_dist
                            {
                                nearest_dist = dist;
                                nearest = Some((pt.0, pt.1));
                                index_in_walls = Some(i);
                                index_in_list = Some(j);
                            }
                        }
                    }

                    match nearest {
                        Some((x, y)) => {
                            let mut u = usize::MAX;
                            let mut v = usize::MAX;
                            match index_in_walls
                            {
                                Some(u_) => {u = u_;},
                                None => {}
                            }
                            match index_in_list
                            {
                                Some(v_)=>{v = v_;},
                                None => {}
                            }
                            current_mode = Mode::WallMid((x, y, Some((u, v))));
                        }
                        None => {current_mode = Mode::WallMid((px, py, None));}
                    }
                }
            },
            Mode::WallMid((sx, sy, inds)) => {
                draw_text("Current Mode: WallMid", 25.0, 100.0, 20.0, RED);
                // Draw a line from (sx, sy) to the current position
                let (px, py) = mouse_position();

                draw_line(sx, sy, px, py, 2.0, BLACK);

                if is_mouse_button_pressed(MouseButton::Left)
                {
                    let (px, py) = mouse_position();
                    match inds
                    {
                        Some(inds) =>
                        {
                            walls[inds.0].insert(inds.1+1, (px, py));
                        }
                        None =>
                        {
                            walls.push(vec![(sx, sy), (px, py)]);
                        }
                    }
                    current_mode = Mode::Idle;
                }
            },
            Mode::StaticObjStart => 
            {
                draw_text("Current Mode: StaticObjStart", 25.0, 100.0, 20.0, RED);
                if is_mouse_button_pressed(MouseButton::Left)
                {
                    let (px, py) = mouse_position();
                    current_mode = Mode::StaticObjMid((px, py));
                }

            },
            Mode::StaticObjMid((sx, sy)) => 
            {
                draw_text("Current Mode: StaticObjMid", 25.0, 100.0, 20.0, RED);
                let (px, py) = mouse_position();

                draw_rectangle(sx, sy, px-sx, py-sy, GRAY);

                if is_mouse_button_pressed(MouseButton::Left)
                {
                    let (px, py) = mouse_position();
                    static_obj.push((sx, sy, px, py));
                    current_mode = Mode::Idle;
                }
            },
        }

        // If right mouse is clicked, then add a new Robot
        if is_mouse_button_pressed(MouseButton::Right)
        {
            let (px, py) = mouse_position();
            robots.push((px, py));
        }

        if is_key_pressed(KeyCode::Escape)
        {
            current_mode = Mode::Idle;
        }

        for line in walls.iter()
        {
            for i in 0..line.len()-1
            {
                let p0 = line[i];
                let p1 = line[i+1];

                // Draw a line for this
                draw_line(p0.0, p0.1, p1.0, p1.1, 2.0, BLACK);
            }
        }

        for rect in static_obj.iter()
        {
            draw_rectangle(rect.0, rect.1, rect.2-rect.0, rect.3-rect.1, GRAY);
        }

        for robot in robots.iter()
        {
            draw_circle(robot.0, robot.1, 10.0, BLACK);
            draw_circle_lines(robot.0, robot.1, 10.0, 2.0, RED);
        }
        // println!("{:?}", current_mode);

        if Some(KeyCode::P) == get_last_key_pressed()
        {
            println!("{:?}", walls);
            println!("**");
        }

        // save
        if Some(KeyCode::S) == get_last_key_pressed()
        {
            let mut robot_configs: Vec<RobotConfig> = Vec::new();
            let mut wall_configs: Vec<WallConfig> = Vec::new();
            let mut static_obj_configs: Vec<StaticObjConfig> = Vec::new();
            for i in 0..robots.len()
            {
                let tfed = tf_function(robots[i]);
                let rc = RobotConfig
                {
                    pose: (tfed.0, tfed.1, 0.0),
                    id: "robot".to_owned() + &i.to_string(),
                    vel: (0.0, 0.0),
                    lidar: true,
                };

                robot_configs.push(rc);
            }

            for wall in walls.iter()
            {
                let mut endpoints: Vec<(f32, f32)> = Vec::new();

                for pt in wall.iter()
                {
                    let tf_ed = tf_function(*pt);
                    endpoints.push(tf_ed);
                }
                wall_configs.push(WallConfig {endpoints});
            }

            for obj in static_obj.iter()
            {
                let (sx, sy, px, py) = obj;
                let width = (sx-px).abs() * RESOLUTION;
                let height = (sy-py).abs() * RESOLUTION;
                let center = tf_function(((sx+px)*0.5, (sy+py)*0.5));

                static_obj_configs.push(StaticObjConfig { center, width, height});
            }

            
            let config = Config
            {
                robots: robot_configs,
                walls: wall_configs,
                static_objects: static_obj_configs,
            };

            let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("new_config.yaml")
            .expect("Couldn't open file");
            serde_yaml::to_writer(f, &config).unwrap();

            println!("Wrote to file `new_config.yaml`");
        }
        next_frame().await;
    };
}