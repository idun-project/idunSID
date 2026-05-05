#!/bin/bash
export GPGKEY=48FF70B8434078A7F830E720D91EF4A55F9D3B3C
CARCH=armv7h makepkg --config makepkg.conf --sign
