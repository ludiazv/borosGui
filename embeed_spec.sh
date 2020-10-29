#!/bin/bash
DST="src/devices.rs"
echo "// This file is generated with to embeed spec yml into static variable" > $DST
echo "pub const yml : &'static str = r#\"" >> $DST
cat spec.yml >> $DST
echo "\"#;" >> $DST
