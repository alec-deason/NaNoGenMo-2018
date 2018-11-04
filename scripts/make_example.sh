#!/bin/sh

FILE_NAME=`date +%Y_%m_%d_%k_%M`.txt

cargo run  > examples/$FILE_NAME
