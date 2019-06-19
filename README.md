
![logo](logo.png)

# Introduction

Bogobarf is a framework for robotic software.

Features:
- pub/sub
- service calls
- events
- nodes
- parameters

Bogobarf is a multi-platform multi-programming language robotic software suite.

# Usage

Bogobarf aims to be a simple client server system. First, start a central server

    cd examples/python
    python bogobarf.py serve

Now you can try the example python scripts:

    cd examples/python
    python talker.py

And in another terminal:

    cd examples/python
    python listener.py

# Design

The bogobarf communication protocol is pretty simple. You open
a TCP connection, and send length delimited packets over this
stream. Length delimited to create discrete packets.

Each packet consist of a CBOR serialized message. There are
several predefined message types:

- rpc (remote procedure call) call
- ret : response from a call
- pub : publish a value
- bye : initiate connection shutdown with the server.

# Rationale

C/C++ is the wrong language for complex software. The low level nature of C/C++ simply slows down development. Programmers
are too busy implementing linked lists, doing memory management, solving memory leaks. This is unnecessary and a waste of time.
Programmers should worry about actual problems.

Instead of unsafe languages, robotic software should be build with modern languages such as Rust, Go, C#, Scala or Python.
Those languages have a lot in common:

- Large eco system of libraries
- Proper development tooling (dependencies, build, test)
- Modern language features

# Other robot software frameworks

Here is a list of other robotic frameworks:

- ROS: http://www.ros.org/
- yarp: http://yarp.it/
