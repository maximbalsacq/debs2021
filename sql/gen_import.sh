#!/bin/bash

for i in {0..9999};
do
	# paths are like <root>/sql/<id div 1000>/<id modulo 1000>.sql
	echo "\\i ${DEBS_DATA_ROOT}/sql/$(($i / 1000))/$(($i % 1000)).sql";
done
#\i /run/media/m/PUBLIC/Thesis/sql/0/0.sql
