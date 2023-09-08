robots:
- id: robot0
  pose:
  - 0.0
  - 0.0
  - 0.0
  vel:
  - 0.0
  - 0.0
  lidar: true
  footprint:
  - 1.2
  - 0.6
walls:
- endpoints:
  - - 3.0
    - 1.0
  - - 3.0
    - 5.0
  - - 10.0
    - 5.0
- endpoints:
  - - -3.0
    - -1.0
  - - -3.0
    - -5.0
  - - -10.0
    - -5.0
- endpoints:
  - - -3.0
    - 1.0
  - - -3.0
    - 5.0
  - - -10.0
    - 5.0
- endpoints:
  - - 3.0
    - -1.0
  - - 3.0
    - -5.0
  - - 10.0
    - -5.0
static_objects:
- center:
  - 10.0
  - 10.0
  width: 3.0
  height: 2.0
- center:
  - -10.0
  - -7.0
  width: 5.0
  height: 2.0
- center:
  - -3.0
  - 12.0
  width: 5.5
  height: 2.0
- center:
  - 13.0
  - -10.0
  width: 3.0
  height: 3.0
