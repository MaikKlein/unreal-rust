echo "Symlinking RustPlugin into RustExample"
mkdir -p example/RustExample/Plugins
ln -s ../../../RustPlugin example/RustExample/Plugins/RustPlugin
mkdir -p example/RustExample/Binaries
