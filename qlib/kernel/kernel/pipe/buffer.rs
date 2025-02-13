// Copyright (c) 2021 Quark Container Authors / 2018 The gVisor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::boxed::Box;
use core::cell::RefCell;
use core::ops::Deref;

use super::super::super::super::common::*;
use super::super::super::super::mem::seq::*;

pub const BUF_SIZE: usize = 32;

/*lazy_static! {
    pub static ref BUF_POOL : Pool<Buffer> = Pool::New(BUF_SIZE);
}*/

pub fn NewBuff() -> Buffer {
    return Buffer::default();
}

pub fn ReturnBuff(_buf: Buffer) {}

// buffer encapsulates a queueable byte buffer.
//
// Note that the total size is slightly less than two pages. This
// is done intentionally to ensure that the buffer object aligns
// with runtime internals. We have no hard size or alignment
// requirements. This two page size will effectively minimize
// internal fragmentation, but still have a large enough chunk
// to limit excessive segmentation.
//

#[derive(Default)]
pub struct Buffer(pub Box<RefCell<BufferIntern>>);

impl Deref for Buffer {
    type Target = Box<RefCell<BufferIntern>>;

    fn deref(&self) -> &Box<RefCell<BufferIntern>> {
        &self.0
    }
}

pub const PIPE_BUF_SIZE: usize = 8144;

#[repr(C)]
pub struct BufferIntern {
    pub data: [u8; PIPE_BUF_SIZE],
    pub read: usize,
    pub write: usize,
}

impl Default for BufferIntern {
    fn default() -> Self {
        return Self {
            data: [0; PIPE_BUF_SIZE],
            read: 0,
            write: 0,
        };
    }
}

impl BufferIntern {
    // Reset resets internal data.
    //
    // This must be called before use.
    pub fn Reset(&mut self) {
        let mut b = self;
        b.read = 0;
        b.write = 0;
    }

    // Empty indicates the buffer is empty.
    //
    // This indicates there is no data left to read.
    pub fn Empty(&self) -> bool {
        let b = self;
        return b.read == b.write;
    }

    // Full indicates the buffer is full.
    //
    // This indicates there is no capacity left to write.
    pub fn Full(&self) -> bool {
        let b = self;
        return b.write == b.data.len();
    }
}

impl Buffer {
    pub fn Count(&self) -> usize {
        let b = self.borrow();
        return b.write - b.read;
    }
}

impl BlockSeqReader for Buffer {
    fn ReadToBlocks(&mut self, dsts: BlockSeq) -> Result<usize> {
        let mut b = self.borrow_mut();
        let n = dsts.CopyOut(&b.data[b.read..b.write]);
        b.read += n;
        return Ok(n);
    }

    fn CopyToBlocks(&self, dsts: BlockSeq) -> Result<usize> {
        let b = self.borrow();
        let n = dsts.CopyOut(&b.data[b.read..b.write]);
        return Ok(n);
    }
}

impl BlockSeqWriter for Buffer {
    fn WriteFromBlocks(&mut self, srcs: BlockSeq) -> Result<usize> {
        let mut b = self.borrow_mut();
        let write = b.write;
        let n = srcs.CopyIn(&mut b.data[write..]);
        b.write += n;
        return Ok(n);
    }
}
