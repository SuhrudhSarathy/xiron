import json
from dataclasses import dataclass
from typing import Tuple, List

@dataclass
class RobotConfig:
    id: str
    pose: Tuple[float, float, float]
    vel: Tuple[float, float]
    lidar: bool = False

    def to_json(self):
        return json.dumps(self.__dict__)
    
    def from_json(self, json):
        self.id = json["id"]
        self.pose = json["pose"]
        self.vel = json["vel"]
        self.lidar = json["lidar"]

@dataclass
class Config:
    robots: List[RobotConfig]
    walls: List
    static_objects: List

    def to_json(self):
        return json.dumps(self.__dict__)
    
@dataclass
class Twist:
    id: str
    vel: Tuple[float, float]

    def to_json(self):
        return json.dumps(self.__dict__)
    
@dataclass
class TwistArray:
    twists: List[Twist]

    def to_json(self):
        my_dict = {}
        my_dict["twists"] = []
        for twist in self.twists:
            my_dict["twists"].append(json.dumps(twist.__dict__))
        
        return json.dumps(my_dict)