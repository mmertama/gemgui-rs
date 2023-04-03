# GemGui

Graphics User Interface library.

GemGui has tested for Windows, Mac OSX and Ubuntu Linux.

GemGui is minimalistic and simple; It is a UI framework without widgets - instead, the UI is composed using common web tools and frameworks. Therefore it is small, easy to learn and quick to take in use.

For the application development the engine is supposed to be implemented using Rust and UI composed using CSS and HTML - like any web front end. GemGui library implements an interface to interact with the UI - the whole API is only a few dozen calls.

GemGui lets write platform independent UI applications with Rust, and combines power of Rust with vast options of front end development tools, sources, documents and frameworks that are only available for Web Developers.

GemGui itself does not contain an application window. The UI by default uses native system browser. However that is fully configurable per application e.g. to utilize Python webview or browser in kiosk-mode.

The Python webview can be installed using Pip - see [PyPi](https://pypi.org/project/pywebview/0.5/)

GemGui is absolute Rust rewrite of [Gempyre C++ GUI Library](https://github.com/mmertama/Gempyre).

Available at [crates.io](https://crates.io/).

See [examples](https://github.com/mmertama/gemgui-rs/tree/main/examples). 

MIT License. 
Copyright Markus Mertama 2023.

