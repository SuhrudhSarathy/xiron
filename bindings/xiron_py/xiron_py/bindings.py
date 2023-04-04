import grpc
from xiron_py.interface_pb2 import VelocityRequest, PoseRequest
from xiron_py.interface_pb2_grpc import XironInterfaceStub
import threading

from time import sleep

class XironPythonInterface:
    def __init__(self):
        self.channel = grpc.insecure_channel("[::1]:8081")
        self.stub = XironInterfaceStub(self.channel)

        self.threads = []

    def set_velocity(self, robot_id, v, w):
        self.stub.SetVelocity(VelocityRequest(id=robot_id, v=v, w=w))

    def get_pose(self, robot_id):
        pose = self.stub.GetPose(PoseRequest(id=robot_id))

        return [pose.x, pose.y, pose.theta]
    
    def add_pose_subscriber(self, robot_id, callback, freq):
        """Adds a subscriber callback to the given robot_id and loops at a frequency"""
        def get_responses(robot_id, callback, timeout):
            try:
                while True:
                    resp = self.stub.GetPose(PoseRequest(id=robot_id))
                    callback([resp.x, resp.y, resp.theta])

                    # Sleep for timeout
                    sleep(1/freq)
            except Exception as e:
                print("Encountered Exception", e)

        sub_thread = threading.Thread(target=get_responses, args=(robot_id, callback, freq))
        sub_thread.daemon = True
        sub_thread.start()

        self.threads.append(sub_thread)

    def spin(self):
        while True:
            sleep(1)

