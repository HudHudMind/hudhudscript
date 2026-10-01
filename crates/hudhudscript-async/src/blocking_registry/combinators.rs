use std::sync::mpsc;

use super::{PromiseRegistry, PromiseResult, RegistryError};

impl<V> PromiseRegistry<V> {
    /// Block until ALL supplied promise ids settle, concurrently.
    ///
    /// Mirrors `Promise.all` semantics: returns `Ok(values)` in input
    /// order if every id resolves, or `Err(RegistryError::Rejected(_))`
    /// as soon as the first rejection arrives (remaining spawned threads
    /// continue running to completion but their results are discarded —
    /// they were already launched by `spawn_task` / the VM's
    /// `spawn_async_chunk`, so we cannot abort them, only ignore their
    /// output).
    ///
    /// Each id's underlying receiver is consumed exactly once. Cached
    /// entries are drained inline and never spawn a thread. Any id that
    /// is neither cached nor registered short-circuits with
    /// `RegistryError::Unregistered`.
    ///
    /// This is the non-tokio counterpart of `combinators::promise_all`,
    /// used by the VM (which is tokio-free by design — see module docs).
    pub fn await_all_blocking(&mut self, ids: &[&str]) -> Result<Vec<V>, RegistryError>
    where
        V: Send + 'static,
    {
        let n = ids.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        if n == 1 {
            let val = self.await_blocking(ids[0])?;
            return Ok(vec![val]);
        }

        let mut slots: Vec<Option<V>> = (0..n).map(|_| None).collect();
        let mut receivers = Vec::new();

        for (idx, id) in ids.iter().enumerate() {
            if let Some(result) = self.take_cached_result(id) {
                match result {
                    Ok(v) => slots[idx] = Some(v),
                    Err(msg) => return Err(RegistryError::Rejected(msg)),
                }
                continue;
            }
            if let Some(receiver) = self.take_receiver(id) {
                receivers.push((idx, (*id).to_string(), receiver));
                continue;
            }
            return Err(RegistryError::Unregistered((*id).to_string()));
        }

        let mut pending_rx = Vec::new();
        for (idx, id, receiver) in receivers {
            match receiver.try_recv() {
                Ok(Ok(val)) => {
                    slots[idx] = Some(val);
                }
                Ok(Err(msg)) => return Err(RegistryError::Rejected(msg)),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(RegistryError::SenderDropped(id));
                }
                Err(mpsc::TryRecvError::Empty) => {
                    pending_rx.push((idx, id, receiver));
                }
            }
        }

        if pending_rx.is_empty() {
            return Ok(slots
                .into_iter()
                .map(|slot| slot.expect("every slot filled when pending is 0"))
                .collect());
        }

        if pending_rx.len() == 1 {
            let (idx, id, receiver) = pending_rx.pop().unwrap();
            match receiver.recv() {
                Ok(Ok(val)) => {
                    slots[idx] = Some(val);
                    return Ok(slots
                        .into_iter()
                        .map(|slot| slot.expect("every slot filled"))
                        .collect());
                }
                Ok(Err(msg)) => return Err(RegistryError::Rejected(msg)),
                Err(_) => return Err(RegistryError::SenderDropped(id)),
            }
        }

        let (agg_tx, agg_rx) = mpsc::channel::<(usize, Result<PromiseResult<V>, RegistryError>)>();
        let mut pending = pending_rx.len();

        for (idx, id, receiver) in pending_rx {
            let tx = agg_tx.clone();
            std::thread::spawn(move || {
                let forwarded = match receiver.recv() {
                    Ok(inner) => Ok(inner),
                    Err(_) => Err(RegistryError::SenderDropped(id)),
                };
                let _ = tx.send((idx, forwarded));
            });
        }
        drop(agg_tx);

        while pending > 0 {
            match agg_rx.recv() {
                Ok((idx, Ok(Ok(val)))) => {
                    slots[idx] = Some(val);
                    pending -= 1;
                }
                Ok((_idx, Ok(Err(msg)))) => {
                    return Err(RegistryError::Rejected(msg));
                }
                Ok((_idx, Err(e))) => {
                    return Err(e);
                }
                Err(_) => {
                    return Err(RegistryError::SenderDropped(
                        "await_all_blocking aggregate".to_string(),
                    ));
                }
            }
        }

        Ok(slots
            .into_iter()
            .map(|slot| slot.expect("every slot filled after pending==0"))
            .collect())
    }

    /// Block until the FIRST supplied promise id settles, concurrently.
    ///
    /// Mirrors `Promise.race` semantics: returns `Ok((idx, value))` for
    /// the earliest resolution, or `Err(RegistryError::Rejected(_))` if
    /// the earliest settlement is a rejection. The remaining spawned
    /// threads continue running to completion — their results are
    /// dropped on the floor, matching "winner takes all". Cached entries
    /// short-circuit immediately (first cached entry wins, in input
    /// order, since there is no real timing for them).
    ///
    /// Non-empty `ids` is required; empty input returns
    /// `RegistryError::Unregistered("race:empty")` so the caller can
    /// translate it into the runtime-appropriate rejection (the VM
    /// currently reports "Promise.race() on empty array").
    ///
    /// This is the non-tokio counterpart of `combinators::promise_race`.
    pub fn await_race_blocking(&mut self, ids: &[&str]) -> Result<(usize, V), RegistryError>
    where
        V: Send + 'static,
    {
        if ids.is_empty() {
            return Err(RegistryError::Unregistered("race:empty".to_string()));
        }

        if ids.len() == 1 {
            let val = self.await_blocking(ids[0])?;
            return Ok((0, val));
        }

        for (idx, id) in ids.iter().enumerate() {
            if let Some(result) = self.take_cached_result(id) {
                return match result {
                    Ok(v) => Ok((idx, v)),
                    Err(msg) => Err(RegistryError::Rejected(msg)),
                };
            }
        }

        let mut receivers = Vec::new();
        for (idx, id) in ids.iter().enumerate() {
            if let Some(receiver) = self.take_receiver(id) {
                receivers.push((idx, (*id).to_string(), receiver));
                continue;
            }
            return Err(RegistryError::Unregistered((*id).to_string()));
        }

        let mut pending_rx = Vec::new();
        for (idx, id, receiver) in receivers {
            match receiver.try_recv() {
                Ok(Ok(val)) => return Ok((idx, val)),
                Ok(Err(msg)) => return Err(RegistryError::Rejected(msg)),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(RegistryError::SenderDropped(id));
                }
                Err(mpsc::TryRecvError::Empty) => {
                    pending_rx.push((idx, id, receiver));
                }
            }
        }

        if pending_rx.is_empty() {
            return Err(RegistryError::Unregistered("race:no-receivers".to_string()));
        }

        if pending_rx.len() == 1 {
            let (idx, id, receiver) = pending_rx.pop().unwrap();
            return match receiver.recv() {
                Ok(Ok(val)) => Ok((idx, val)),
                Ok(Err(msg)) => Err(RegistryError::Rejected(msg)),
                Err(_) => Err(RegistryError::SenderDropped(id)),
            };
        }

        let (agg_tx, agg_rx) = mpsc::channel::<(usize, Result<PromiseResult<V>, RegistryError>)>();

        for (idx, id, receiver) in pending_rx {
            let tx = agg_tx.clone();
            std::thread::spawn(move || {
                let forwarded = match receiver.recv() {
                    Ok(inner) => Ok(inner),
                    Err(_) => Err(RegistryError::SenderDropped(id)),
                };
                let _ = tx.send((idx, forwarded));
            });
        }
        drop(agg_tx);

        match agg_rx.recv() {
            Ok((idx, Ok(Ok(val)))) => Ok((idx, val)),
            Ok((_idx, Ok(Err(msg)))) => Err(RegistryError::Rejected(msg)),
            Ok((_idx, Err(e))) => Err(e),
            Err(_) => Err(RegistryError::SenderDropped(
                "await_race_blocking aggregate".to_string(),
            )),
        }
    }
}
