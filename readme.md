# contacts on all sides
## warning.
This code is an absolute mess, especially on the server side, I do not recommend this to be used as learning material in any circumstances.
this project is abandenware, I might mess with it from time to time, but that's very unlikely, do not expect help from me beyond this readme file.
## note about assets.
nothing from the game is in the sounds folder, in fact, all the files in the sounds folder are just place holders to make it easier to copy and paste files, the reason for this is because many of the assets I'm using were purchased, so I had to remove them to not violate the license.
right now, opening the game will probably throw an error your way since a sound cannot be found, It shouldn't be like that, but the sound handling code I used was made by me 2021 version and I never gotten to using proper errors instead of unwrap, so if you want to avoid that just modify src/sound.rs.
## building.
### windows(client)
to build you would be required to install and do a phew things first.
1. Install llvm, you can get it from [here](https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.1/LLVM-17.0.1-win64.exe), click on the add to environment variables and install.
2. You need to get openssl, following [this](https://docs.rs/crate/openssl-sys/0.9.19) link, under windows, should get you setuped.
3. Get protoc, you can get it from [here](https://github.com/protocolbuffers/protobuf/releases/download/v25.2/protoc-25.2-win64.zip), After downloading, extract the zip and add it to the windows path.
4. Remove the 260 characters windows path limitations, you can follow [this](https://www.howtogeek.com/266621/how-to-make-windows-10-accept-file-paths-over-260-characters/), if you have python3 installed, you probably already have this option enabled because python3's setup automatically asks you if you wish to remove that limitation, you could also use the python3 installer if you don't understand how the guide works.

## server.
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev libclang-dev protobuf-compiler