//! BULGU6: try_frames sahiplik yardımcıları.
//!
//! TF kayıtları (catch_ip, iter, loop, owner_serial) VM-geneldir; bir TF'i
//! yalnız SAHİBİ olan çerçeve tüketebilir. Anahtar SERIAL'dir — call_depth
//! pop'ta düştüğü için ayırt edici değildir (v0.9.54 regresyonunun dersi).
use crate::vm::machine::VM;

/// Yığın tepesindeki TF güncel çerçeveye aitse çekip döndürür; ait değilse
/// hiç dokunmaz (istisna sahip çerçeveye kadar taşar).
impl VM {
    #[inline]
    pub(crate) fn pop_own_try_frame(&mut self) -> Option<(usize, usize, usize, usize)> {
        let owner_ok = match (self.try_frames.last(), self.frame_stack.last()) {
            (Some((_, _, _, owner)), Some(frame)) => *owner == frame.serial,
            _ => false,
        };
        if owner_ok {
            self.try_frames.pop()
        } else {
            None
        }
    }
}
