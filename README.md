![Pipec Logo](https://github.com/jessie-framework/pipeclogo/blob/main/pipeclogonobackground.png?raw=true)

# About

Pipec is a compiled,typed low level language with the goal of making faster graphical applications easier to write.

## How It Works

Pipec source code translates into machine code using LLVM and is then linked with a graphical API of your choice (eg. OpenGL, Vulkan). The language provides an interface to interact with the graphics part of your app called viewports which you can then decide how you are going to present them (saving into an image, presenting onto a window, etc.)

## Why Pipec?

Because while traditional renderers which adapt to your programs needs at runtime may be a tempting choice because of their API simplicity, they also leave out a lot of very much needed control. Well written code using a low level graphics API will most likely be faster than code using a drawing library . But even still , it is low level code, and hard for a regular developer to orchestrate correctly. The goal of Pipec is to give the developer just enough control for them to understand what their code is going to do behind the scenes, and just enough magic to make them have the fastest code possible.

Here are some things possible by orchestrating low level graphics API code ahead of time :

1. Needless shaders are eliminated from the binary
2. Every pipeline your program needs can be uploaded at the beginning of runtime rather than active prediction made by a library resulting in fewer frame time and smoother performance
3. Static parts of your program can be uploaded to memory at the beginning of runtime
4. Geometry can be optimized ahead of time
5. Instancing can be done ahead of time
6. And many more!

And these can be done for YOUR program whether its a simple UI application or a complex game. The possibilities are endless!

## Contributing

Any contributions to the project are welcome. You can start by cloning the repository and then opening a new pull request.

Pipec source code is free and open source published under the MIT license.

