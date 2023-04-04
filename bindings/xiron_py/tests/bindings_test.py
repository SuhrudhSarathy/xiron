from xiron_py import XironPythonInterface

def pose_callback_funtion(msg):
    print(msg)

if __name__ == '__main__':
    xip = XironPythonInterface()
    xip.set_velocity("robot0", 0.25, 0.3)

    xip.add_pose_subscriber("robot0", pose_callback_funtion, 10)

    xip.spin()