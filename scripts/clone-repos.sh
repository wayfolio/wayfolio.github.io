#!/bin/bash

set -ex

cd "$(dirname "$BASH_SOURCE")/.."

if [[ "$1" = "CI" ]]; then
  function clone() {
    git clone --depth 1 --revision $3 $1 submodules/$2
  }
else
  function clone() {
    git clone $1 submodules/$2
    git -C submodules/$2 checkout $3
  }
fi

clone https://github.com/mahkoh/wayland-db.git \
      wayland-db \
      HEAD

clone https://github.com/wayfolio/compositor-support.git \
      compositor-support \
      HEAD

clone https://github.com/wayfolio/wayfolio.git \
      wayfolio \
      HEAD # todo
