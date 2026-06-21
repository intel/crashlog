#!/bin/sh
# Copyright (C) 2026 Intel Corporation
# SPDX-License-Identifier: MIT
#
# Extracts the Crash Log data from the PMT device given as argument.
#
# Usage: iclg-extract.sh <device>    (e.g. iclg-extract.sh crashlog0)
#
# Customization:
#   - Change ICLG or OUTPUT_PATH if iclg or the output directory is located
#     elsewhere.
#   - Additional control commands can be added after the extraction, such as
#     rearming the device that just completed its Crash Log collection.
#     Do not trigger a Crash Log collection on the same device from this
#     script, as its completion would start this script again in a loop.

set -eu

ICLG=/usr/local/bin/iclg
OUTPUT_PATH=/var/log/crashlog/

if [ $# -ne 1 ]; then
    echo "Usage: $0 <device>" >&2
    exit 1
fi

DEVICE="$1"

FILES=$("$ICLG" -vv extract -s "pmt:$DEVICE" "$OUTPUT_PATH")

if [ -z "$FILES" ]; then
    echo "No Crash Log extracted from $DEVICE" >&2
    exit 1
fi

echo "Extracted: $FILES"

# Uncomment to rearm the device:
# "$ICLG" -vv rearm -s "pmt:$DEVICE"
# echo "Rearmed $DEVICE"
