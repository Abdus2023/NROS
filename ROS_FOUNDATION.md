# ROS Foundation & NROS Proposition

**Robot Operating System** (**ROS** or **ros**) is an [open-source](https://en.wikipedia.org/wiki/Open-source_software) [robotics middleware](https://en.wikipedia.org/wiki/Robotics_middleware) suite. Although ROS is not an [operating system](https://en.wikipedia.org/wiki/Operating_system) (OS) but a set of [software frameworks](https://en.wikipedia.org/wiki/Software_framework) for [robot software](https://en.wikipedia.org/wiki/Robot_software) [development](https://en.wikipedia.org/wiki/Software_development), it provides services designed for a heterogeneous [computer cluster](https://en.wikipedia.org/wiki/Computer_cluster) such as [hardware abstraction](https://en.wikipedia.org/wiki/Hardware_abstraction), low-level [device control](https://en.wikipedia.org/wiki/Device_driver), implementation of commonly used functionality, [message-passing between processes](https://en.wikipedia.org/wiki/Inter-process_communication), and [package management](https://en.wikipedia.org/wiki/Package_manager). Running sets of ROS-based processes are represented in a [graph](https://en.wikipedia.org/wiki/Graph_theory) architecture where processing takes place in nodes that may receive, post, and [multiplex](https://en.wikipedia.org/wiki/Multiplexing) sensor data, control, state, planning, actuator, and other messages.

Despite the importance of reactivity and [low latency](https://en.wikipedia.org/wiki/Low_latency) in robot control, ROS is not a [real-time operating system](https://en.wikipedia.org/wiki/Real-time_operating_system) (RTOS). However, it is possible to integrate ROS with [real-time computing](https://en.wikipedia.org/wiki/Real-time_computing) code.[3] The lack of support for real-time systems has been addressed in the creation of ROS 2,[4][5][6] a major revision of the ROS API which will take advantage of modern libraries and technologies for core ROS functions and add support for real-time code and [embedded system](https://en.wikipedia.org/wiki/Embedded_system) hardware.

Software in the ROS [ecosystem](https://en.wikipedia.org/wiki/Software_ecosystem)[7] can be separated into three groups:

- language- and platform-independent tools used for building and distributing ROS-based software;
- ROS client library implementations such as roscpp,[8] rospy,[9] and roslisp;[10]
- packages containing application-related code that uses one or more ROS client libraries.[11]

Both the language-independent tools and the main client libraries ([C++](https://en.wikipedia.org/wiki/C++), [Python](https://en.wikipedia.org/wiki/Python_(programming_language)), and [Lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language))) are released under the terms of the [BSD license](https://en.wikipedia.org/wiki/BSD_license), and as such are [open-source software](https://en.wikipedia.org/wiki/Open-source_software) and free for both commercial and research use. The majority of other packages are licensed under a variety of [open-source licenses](https://en.wikipedia.org/wiki/Open-source_license). These other packages implement commonly used functionality and applications such as hardware drivers, robot models, datatypes, planning, [perception](https://en.wikipedia.org/wiki/Robotic_sensing), [simultaneous localization and mapping](https://en.wikipedia.org/wiki/Simultaneous_localization_and_mapping) (SLAM), [simulation tools](https://en.wikipedia.org/wiki/Robotics_simulator), and other [algorithms](https://en.wikipedia.org/wiki/Algorithm).

The main ROS client libraries are geared toward a [Unix-like](https://en.wikipedia.org/wiki/Unix-like) system, mostly because of their dependence on large sets of open-source software dependencies. For these client libraries, [Ubuntu Linux](https://en.wikipedia.org/wiki/Ubuntu_(operating_system)) is listed as "Supported" while other variants such as [Fedora Linux](https://en.wikipedia.org/wiki/Fedora_Linux), [macOS](https://en.wikipedia.org/wiki/MacOS), and [Microsoft Windows](https://en.wikipedia.org/wiki/Microsoft_Windows) are designated "experimental" and are supported by the community.[12] The native Java ROS client library, rosjava,[13] however, does not share these limitations and has enabled ROS-based software to be written for the [Android OS](https://en.wikipedia.org/wiki/Android_(operating_system)).[14] rosjava has also enabled ROS to be integrated into an officially supported [MATLAB](https://en.wikipedia.org/wiki/MATLAB) toolbox which can be used on [Linux](https://en.wikipedia.org/wiki/Linux), macOS, and Microsoft Windows.[15] A [JavaScript](https://en.wikipedia.org/wiki/JavaScript) client library, roslibjs[16] has also been developed which enables integration of software into a ROS system via any standards-compliant web browser.

ROS was designed to be open source, intending that users would be able to choose the configuration of tools and libraries that interacted with the core of ROS so that users could shift their software stacks to fit their robot and application area. As such, there is very little which is core to ROS, beyond the general structure within which programs must exist and communicate. In one sense, ROS is the underlying plumbing behind nodes and message passing. However, in reality, ROS is not only plumbing, but a rich and mature set of tools, a wide-ranging set of robot-agnostic abilities provided by packages, and a greater ecosystem of additions to ROS.

### Computation graph model

ROS processes are represented as nodes in a graph structure, connected by edges called topics.[66] ROS nodes can pass messages to one another through topics, make service calls to other nodes, provide a service for other nodes, or set or retrieve shared data from a communal database called the parameter server. A process called the ROS1 Master[66] makes all of this possible by registering nodes to themselves, setting up node-to-node communication for topics, and controlling parameter server updates. Messages and service calls do not pass through the master, rather the master sets up peer-to-peer communication between all node processes after they register themselves with the master. This decentralized architecture lends itself well to robots, which often consist of a subset of networked computer hardware, and may communicate with off-board computers for heavy computing or commands.

#### Nodes

A node represents one process running the ROS graph. Every node has a name, which registers with the ROS1 master before it can take any other actions. Multiple nodes with different names can exist under different [namespaces](https://en.wikipedia.org/wiki/Namespace), or a node can be defined as anonymous, in which case it will randomly generate an additional identifier to add to its given name. Nodes are at the center of ROS programming, as most ROS client code is in the form of a ROS node which takes actions based on information received from other nodes, sends information to other nodes, or sends and receives requests for actions to and from other nodes.

#### Topics

Topics are named [buses](https://en.wikipedia.org/wiki/Software_bus) over which nodes send and receive messages.[67] Topic names must be unique within their namespace as well. To send messages to a topic, a node must publish to said topic, while to receive messages it must subscribe. The publish/subscribe model is anonymous: no node knows which nodes are sending or receiving on a topic, only that it is sending/receiving on that topic. The types of messages passed on a topic vary widely and can be user-defined. The content of these messages can be sensor data, motor control commands, state information, actuator commands, or anything else.

#### Services

A node may also advertise services.[68] A service represents an action that a node can take which will have a single result. As such, services are often used for actions that have a defined start and end, such as capturing a one-frame image, rather than processing velocity commands to a wheel motor or odometer data from a wheel encoder. Nodes advertise services and call services from one another.

#### Parameter server

The parameter server[68] is a database shared between nodes which allows for communal access to static or semi-static information. Data that does not change frequently and as such will be infrequently accessed, such as the distance between two fixed points in the environment, or the weight of the robot, are good candidates for storage in the parameter server.

ROS's core functionality is augmented by a variety of tools that allow developers to visualize and record data, easily navigate the ROS package structures, and create scripts automating complex configuration and setup processes. The addition of these tools greatly increases the abilities of systems using ROS by simplifying and providing solutions to several common robotics development problems. These tools are provided in packages like any other algorithm, but rather than providing implementations of hardware drivers or algorithms for various robotic tasks, these packages provide task and robot-agnostic tools that come with the core of most modern ROS installations.

### rviz

rviz[69] (Robot Visualization tool) is a three-dimensional visualizer used to visualize robots, the environments they work in, and sensor data. It is a highly configurable tool, with many different types of visualizations and plugins. Unified Robot Description Format ([URDF](https://en.wikipedia.org/wiki/URDF)) is an [XML](https://en.wikipedia.org/wiki/XML) file format for robot model description.

### rosbag

rosbag[70] is a command line tool used to record and playback ROS message data. rosbag uses a file format called bags,[71] which log ROS messages by listening to topics and recording messages as they come in. Playing messages back from a bag is largely the same as having the original nodes that produced the data in the ROS computation graph, making bags a useful tool for recording data to be used in later development. While rosbag is a command line only tool, rqt_bag[72] provides a GUI interface to rosbag.

### catkin

catkin[73] is the ROS1 build system, having replaced rosbuild[74] as of ROS Groovy. catkin is based on [CMake](https://en.wikipedia.org/wiki/CMake) and is similarly cross-platform, open-source, and language-independent. As of ROS2 catkin is no longer in use, but still maintained for legacy support.[75]

### rosbash

The rosbash[76] package provides a suite of tools which augment the functionality of the [bash shell](https://en.wikipedia.org/wiki/Bash_(Unix_shell)). These tools include rosls, roscd, and roscp, which replicate the functionalities of [ls](https://en.wikipedia.org/wiki/Ls), [cd](https://en.wikipedia.org/wiki/Cd_(command)), and [cp](https://en.wikipedia.org/wiki/Cp_(Unix)) respectively. The ROS versions of these tools allow users to use ros package names in place of the file path where the package is located. The package also adds tab-completion to most ROS utilities and includes rosed, which edits a given file with the chosen default text editor, as well rosrun, which runs executables in ROS packages. rosbash supports the same functionalities for [zsh](https://en.wikipedia.org/wiki/Z_shell) and [tcsh](https://en.wikipedia.org/wiki/Tcsh), to a lesser extent.

### roslaunch

roslaunch[77] is a tool used to launch multiple ROS nodes both locally and remotely, as well as setting parameters on the ROS parameter server. roslaunch configuration files, which are written using [XML](https://en.wikipedia.org/wiki/XML) can easily automate a complex startup and configuration process into a single command. roslaunch scripts can include other roslaunch scripts, launch nodes on specific machines, and even restart processes that die during execution.

ROS contains many open-source implementations of common robotics functionality and algorithms. These open-source implementations are organized into packages. Many packages are included as part of ROS distributions, while others may be developed by individuals and distributed through code-sharing sites such as github. Some packages of note include:

### Systems and tools

- *actionlib*[78] provides a standardized interface for interfacing with preemptable tasks.
- *nodelet*[79] provides a way to run multiple algorithms in a single process.
- *rosbridge*[80] provides a JSON API to ROS functionalities for non-ROS programs.

### Mapping and localization

- *slam toolbox*[81] provides full 2D [simultaneous localization and mapping](https://en.wikipedia.org/wiki/Simultaneous_localization_and_mapping) (SLAM) and localization system.
- *gmapping*[82] provides a wrapper for [OpenSlam's](https://en.wikipedia.org/wiki/OpenSlam?action=edit&redlink=1) [Gmapping](https://en.wikipedia.org/wiki/Gmapping?action=edit&redlink=1) algorithm for SLAM.
- *cartographer*[83] provides real-time 2D and 3D SLAM algorithms developed at [Google](https://en.wikipedia.org/wiki/Google).
- *amcl*[84] provides an implementation of [adaptive Monte-Carlo localization.](https://en.wikipedia.org/wiki/Monte_Carlo_localization)

### Navigation

- *navigation*[85] provides the capability of navigating a mobile robot in a planar environment.

### Manipulation

- *MoveIt!*[86] provides motion planning capabilities for [robot manipulators](https://en.wikipedia.org/wiki/Manipulator_(device)). Its default planning library is the [Open Motion Planning Library (OMPL)](https://ompl.kavrakilab.org/).[87]

### Perception

- *vision_opencv*[88] is a meta-package which provides packages for integrating ROS with [OpenCV](https://en.wikipedia.org/wiki/OpenCV).

### Coordinate frame representation

- *tf*[89] provided a system for representing, tracking and transforming coordinate frames until ROS Hydro, when it was deprecated in favor of tf2.
- *tf2*[90] is the second generation of the tf library, and provides the same abilities for ROS versions after Hydro.

### Simulation

- *gazebo_ros_pkgs*[91] is a meta-package which provides packages for integrating ROS with the [Gazebo simulator](https://en.wikipedia.org/wiki/Gazebo_simulator).
- *stage*[92] provides an interface for the 2D [Stage simulator](https://en.wikipedia.org/wiki/Player_Project).

---

This description captures the **ROS 1 architecture** well, but it mixes ROS 1 concepts with ROS 2 and contains several historical statements that should be separated if you're using it as technical reference.

The most important conceptual distinction is:

**ROS is middleware/framework infrastructure, not an operating system.**

A useful architectural decomposition is:

```text
┌─────────────────────────────────────────────────────────────┐
│                    ROBOT APPLICATIONS                       │
│  Navigation │ Manipulation │ Perception │ SLAM │ Planning │
├─────────────────────────────────────────────────────────────┤
│                    ROS PACKAGES                             │
│  Drivers │ Algorithms │ Robot Models │ Interfaces │ Tools  │
├─────────────────────────────────────────────────────────────┤
│                  ROS COMMUNICATION                           │
│  Topics │ Services │ Actions │ Parameters │ Discovery       │
├─────────────────────────────────────────────────────────────┤
│                 ROS CLIENT LIBRARIES                         │
│       rclcpp / rclpy / other language bindings              │
├─────────────────────────────────────────────────────────────┤
│             DDS / RMW / TRANSPORT LAYER                     │
│       Discovery │ QoS │ Serialization │ Networking          │
├─────────────────────────────────────────────────────────────┤
│                     HOST OS                                  │
│       Linux │ Windows │ macOS │ RTOS / Embedded targets     │
├─────────────────────────────────────────────────────────────┤
│                    HARDWARE                                 │
│       CPU │ GPU │ Sensors │ Actuators │ Networks            │
└─────────────────────────────────────────────────────────────┘
```

### ROS 1 vs ROS 2

The text you've supplied is predominantly describing **ROS 1**.

| Concept | ROS 1 | ROS 2 |
|---|---|---|
| Discovery | ROS Master + XMLRPC | DDS-based discovery |
| Core communication | ROS transport | DDS/RTPS through RMW |
| Topics | ✓ | ✓ |
| Services | ✓ | ✓ |
| Actions | `actionlib` | First-class ROS 2 actions |
| Parameters | Central parameter server | Node-local parameter model |
| QoS | Limited | Extensive DDS QoS |
| Real-time | Not designed as RT system | Designed with real-time support in mind |
| Build | catkin | ament + colcon |
| C++ API | roscpp | rclcpp |
| Python API | rospy | rclpy |
| Launch | XML | Python/XML/YAML |
| Node composition | nodelets | composable nodes |
| Embedded/RTOS | Limited | substantially better support |
| Security | Limited | DDS-Security integration |

One particularly important correction is this statement:

> "ROS processes are represented as nodes ... connected by edges called topics."

That's a useful **ROS 1 conceptual simplification**, but technically topics are communication interfaces, not simply graph edges. A ROS graph can contain publishers, subscriptions, services, clients, actions, parameters, and discovery relationships.

Likewise, the **ROS 1 Master** should not be generalized to ROS itself. ROS 2 deliberately removed the ROS Master architecture and uses DDS discovery.

### The deeper architectural idea

ROS can be understood as a **robotic distributed-systems middleware**.

Its fundamental abstraction is:

```text
                ┌───────────────┐
                │     Node      │
                │               │
                │  computation  │
                └───────┬───────┘
                        │
             ┌──────────┼──────────┐
             │          │          │
          publish     service    action
             │          │          │
             ▼          ▼          ▼
          Topic      Service     Action
             │          │          │
             ▼          ▼          ▼
        ┌─────────┐ ┌─────────┐ ┌─────────┐
        │Subscriber│ │ Client  │ │ Server  │
        └─────────┘ └─────────┘ └─────────┘
```

This is why ROS became so powerful: it separates **robot computation** from the mechanisms required to connect that computation.

For example:

```text
Camera Driver
      │
      │ Image messages
      ▼
Image Processing
      │
      │ detected objects
      ▼
Perception
      │
      │ world state
      ▼
Planning
      │
      │ trajectory
      ▼
Controller
      │
      │ actuator commands
      ▼
Robot Hardware
```

Each component can be independently developed, replaced, tested, simulated, distributed across machines, or implemented in different languages.

### ROS's major conceptual layers

I'd therefore model the ecosystem as roughly:

**1. Computation**

- Nodes
- Components
- Executors
- Callback processing

**2. Communication**

- Topics
- Services
- Actions
- Parameters
- Events

**3. Distributed systems**

- Discovery
- Serialization
- Transport
- QoS
- Networking

**4. Robot semantics**

- TF / TF2
- URDF
- Robot state
- Sensors
- Actuators
- Frames

**5. Algorithms**

- SLAM
- Navigation
- Perception
- Planning
- Control
- Manipulation

**6. Development infrastructure**

- Build systems
- Package management
- Launch
- CLI
- Visualization
- Logging
- Recording/playback
- Simulation

### And this distinction is especially important

ROS itself does **not** provide the deterministic control loop that an industrial robot controller or RTOS normally provides.

A typical architecture is closer to:

```text
                High-level autonomy
                       │
                 ROS 2 nodes
                       │
              commands / trajectories
                       │
              ┌────────▼────────┐
              │ Real-time layer │
              │                 │
              │ RT controller   │
              │ RT executor     │
              │ EtherCAT/CAN    │
              └────────┬────────┘
                       │
                 Servo/control
                       │
              ┌────────▼────────┐
              │ Robot hardware  │
              └─────────────────┘
```

That distinction matters enormously for **industrial robotics**: ROS/ROS 2 can provide the orchestration, perception, planning, coordination and higher-level control, while a dedicated deterministic control layer handles hard real-time actuator loops.

So, if you're using this material as the foundation for studying or designing a **ROS-like architecture**, the most useful abstraction is:

> **ROS is a distributed robotic middleware ecosystem whose core purpose is to provide standardized computation, communication, discovery, tooling, and robot-oriented interfaces—not to replace the host OS or necessarily provide the hard-real-time control layer.**

---

If by **NROS** you mean your **NROS repository/project**, then the interesting direction is not simply "rewrite ROS in Rust." It is to treat ROS as the architectural predecessor and ask:

> **What would a ROS-like middleware look like if it were designed today around Rust, deterministic execution, safety, embedded systems, and agentic workloads?**

### ROS → NROS conceptual evolution

```text
ROS
│
├── Nodes
├── Topics
├── Services
├── Actions
├── Parameters
├── ROS Graph
├── DDS / transport
├── Executors
├── Packages
├── Launch
├── Bags / recording
├── TF / robot state
└── Simulation / tooling
        │
        │ redesign
        ▼
NROS
│
├── Components / Actors
├── Typed channels
├── Request / Response
├── Actions / Tasks
├── State & configuration
├── Runtime graph
├── Transport abstraction
├── Deterministic scheduler
├── Rust-native crates
├── Declarative orchestration
├── Event / trace recording
├── Resource & capability model
└── Embedded / RT integration
```

The key change is **architectural rather than linguistic**.

### 1. ROS Node → NROS Component

ROS traditionally makes the **node** the fundamental computational unit.

NROS can make the unit more explicit:

```text
NROS Component
    │
    ├── Inputs
    ├── Outputs
    ├── Requests
    ├── Responses
    ├── Actions
    ├── State
    ├── Resources
    └── Lifecycle
```

A component becomes a typed participant in the runtime rather than simply a process registered in a graph.

### 2. ROS Topic → Typed NROS Channel

Instead of thinking primarily in terms of anonymous topic buses:

```text
Publisher ─── Topic ───> Subscriber
```

NROS can model communication as typed channels:

```text
Producer<T>
     │
     ▼
Channel<T>
     │
     ▼
Consumer<T>
```

Rust then gives NROS an opportunity to make message contracts substantially stronger at compile time.

### 3. ROS Master/Graph → Runtime

ROS 1 has:

```text
             ROS Master
             /   |   \
            /    |    \
         Node  Node   Node
```

NROS can instead have a runtime-oriented model:

```text
              NROS Runtime
                   │
        ┌──────────┼──────────┐
        │          │          │
     Component  Component  Component
        │          │          │
        └────── Channels ─────┘
```

Discovery, scheduling, lifecycle, resources and communication can therefore become coordinated parts of one runtime model.

### 4. ROS Executor → NROS Scheduler

This is potentially one of the biggest opportunities.

ROS traditionally revolves around callbacks and executors.

NROS could make scheduling explicit:

```text
                Scheduler
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
     Sensor      Planner     Controller
      1 kHz         20 Hz       1 kHz
        │           │           │
        └───────────┴───────────┘
                    │
              deterministic
                execution
```

That opens the door to:

- priorities
- deadlines
- budgets
- affinity
- periodic execution
- event-driven execution
- real-time classes
- resource constraints
- deterministic replay

### 5. ROS Package → NROS Crate

ROS:

```text
package
 ├── nodes
 ├── messages
 ├── services
 ├── launch
 └── dependencies
```

NROS:

```text
crate
 ├── components
 ├── messages
 ├── channels
 ├── services
 ├── actions
 ├── runtime integration
 └── Cargo dependencies
```

This gives NROS direct access to the Rust ecosystem:

```text
Cargo
 │
 ├── crates.io
 ├── workspace
 ├── features
 ├── dependency resolution
 ├── rustc
 ├── rustfmt
 └── clippy
```

### 6. ROS message passing → NROS protocol

ROS's message model becomes something more fundamental:

```text
Message
   │
   ├── Type
   ├── Schema
   ├── Version
   ├── Encoding
   ├── QoS
   └── Metadata
```

NROS could therefore treat communication contracts as first-class protocol objects.

For example:

```text
Message<T>

T = LaserScan
T = Pose
T = JointState
T = Trajectory
T = SensorEvent
T = AgentCommand
```

### 7. ROS tools → NROS observability

ROS has tools such as:

- `rosnode`
- `rostopic`
- `rosservice`
- `rosbag`
- `rviz`
- `rqt`

NROS can evolve this into a unified runtime observability model:

```text
nros
 ├── graph
 ├── node/component
 ├── topic/channel
 ├── service
 ├── action
 ├── state
 ├── trace
 ├── record
 ├── replay
 ├── inspect
 └── diagnose
```

The important idea is that **observability becomes part of the runtime contract**, rather than a collection of loosely coupled utilities.

## The deeper NROS proposition

The most interesting interpretation of NROS is therefore:

```text
                 ROS
                  │
       distributed robotics middleware
                  │
                  ▼
                 NROS
                  │
       ┌──────────┼──────────┐
       │          │          │
     Safety    Real-time   Systems
       │          │          │
       └──────────┼──────────┘
                  │
                  ▼
           Rust-native runtime
                  │
                  ▼
       deterministic computation
                  │
                  ▼
        robotics + autonomous systems
```

So the evolution is not:

**ROS → Rust ROS**

but rather:

**ROS → a new runtime model for robotic/autonomous computation.**

And that gives NROS a much stronger identity:

> **NROS = a Rust-native, safety-oriented, deterministic distributed runtime for robotic and autonomous systems.**

That is the architectural lens I would use when analyzing the NROS repository: **map every ROS primitive to its NROS equivalent, then identify which primitives should be preserved, redesigned, or deliberately eliminated.**
