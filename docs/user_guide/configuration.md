# Configuring the Simulator
The configuration of the Simulator's state is done through the use of YAML files. Below is an example of a sample YAML file.

```yaml
robots:
- id: robot0
  pose:
  - -6.295703
  - 7.076855
  - 0.0
  vel:
  - 0.0
  - 0.0
  lidar: true
  footprint:
  - 0.25
  drive_type: Differential
  add_noise: false
- id: robot1
  pose:
  - -4.0486326
  - 4.9811525
  - 0.0
  vel:
  - 0.0
  - 0.0
  lidar: true
  footprint:
  - 0.25
  drive_type: Differential
  add_noise: false
walls:
- endpoints:
  - - -9.1560545
    - 8.704004
  - - -1.0940428
    - 8.6789055
  - - -0.99091816
    - 3.9499025
  - - -9.038574
    - 4.1187496
- endpoints:
  - - 1.0652351
    - 8.664355
  - - 1.0892582
    - 4.006152
  - - 9.079687
    - 4.0520506
static_objects:
- center:
  - 2.2917004
  - 6.981738
  width: 0.625
  height: 0.625
  rotation: 0.0
- center:
  - 7.0378914
  - 6.6941404
  width: 0.625
  height: 0.625
  rotation: 0.0
- center:
  - 0.020019531
  - 2.6319332
  width: 0.625
  height: 0.625
  rotation: 0.0

```
## Three categories of objects
There are three main categories of Objects used in the Simulator, `Robot`, `Static Object`, `Wall`. The configuration for each of these is described differently.

In general, the YAML can be split as a combination of these categories as below

```yaml
robots:
    - ...
    - ...
    - ...
walls:
    - ...
    - ...
    - ...
static_objects:
    - ...
    - ...
    - ...
```

## Configuring a Robot

| Property       | Description                                                                                          | Data Type        |
|---------------|------------------------------------------------------------------------------------------------------|-----------------|
| `id`          | Describes the `id` of the Robot.                                                                     | `string`        |
| `pose`        | Describes the `pose` of the Robot. This is a tuple of [`x`, `y`, `yaw`].                            | Tuple of [`x`, `y`, `yaw`] |
| `vel`         | Describes the `velocity` of the Robot. Varies according to drive type:<br>Differential drive: `[vx, vy]`<br>Omnidirectional drive: velocity in x and y direction<br>Ackermann drive: linear velocity and steering angle. | Varies according to drive type:<br>Differential drive: `[vx, vy]`<br>Omnidirectional drive: velocity in x and y direction<br>Ackermann drive: linear velocity and steering angle. |
| `lidar`       | Describes the presence of a lidar.                                                                     | `bool`          |
| `footprint`   | Describes the footprint. If `float`, the shape of the robot is circular with the given number as radius. If a tuple of `float`, the first number is taken as width and the second number as height. | `float` or Tuple of `float` |
| `drive_type`  | Describes the drive type. Can be `Differential`, `OmniDirectional`, or `Ackermann`.                | `string`        |
| `add_noise`   | Adds noise to the kinematics model.                                                                   |          `bool`       |

An example YAML configuration with `Ackermann` drive and Rectangular Footprint is given below
```yaml

- id: robot1
  pose:
    - -4.825901
    - 8.391234
    - 0.35
  vel:
    - 1.5
    - 0.0
  lidar: true
  footprint:
    - 0.4
    - 1.2
  drive_type: Ackermann
  add_noise: true

```


## Configuring a Static Object

Here's a table generated from the YAML data:

| Property         | Description                                           | Data Type          |
|-----------------|-------------------------------------------------------|--------------------|
| static_objects  | List of static objects                               | List of objects    |
| center          | Center coordinates of the object                     | Tuple of [`x`, `y`] |
| width           | Width of the object                                  | `float`            |
| height          | Height of the object                                 | `float`            |
| rotation        | Rotation of the object                               | `float`            |

In this table, I've represented the YAML data as a table with property names, descriptions, and data types for each field.

## Configuring a Wall

| Property    | Description                               | Data Type          |
|------------|-------------------------------------------|--------------------|
| endpoints  | List of endpoint coordinates               | List of Tuples of XY coordinates of vertices     |

### Note
It is highly recommended to create a starting configuration using the [GUI](./gui_usage.md) and edit the specifics from the YAML file.
