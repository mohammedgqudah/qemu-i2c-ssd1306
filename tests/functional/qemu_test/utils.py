# Utilities for python-based QEMU tests
#
# Copyright 2024 Red Hat, Inc.
#
# Authors:
#  Thomas Huth <thuth@redhat.com>
#
# This work is licensed under the terms of the GNU GPL, version 2 or
# later.  See the COPYING file in the top-level directory.

import os

"""
Round up to next power of 2
"""
def pow2ceil(x):
    return 1 if x == 0 else 2**(x - 1).bit_length()

def file_truncate(path, size):
    if size != os.path.getsize(path):
        with open(path, 'ab+') as fd:
            fd.truncate(size)

"""
Expand file size to next power of 2
"""
def image_pow2ceil_expand(path):
        size = os.path.getsize(path)
        size_aligned = pow2ceil(size)
        if size != size_aligned:
            with open(path, 'ab+') as fd:
                fd.truncate(size_aligned)
