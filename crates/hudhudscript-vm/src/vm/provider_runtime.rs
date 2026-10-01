//! M1: kalıcı paylaşılan tokio runtime async→sync köprüsü — split from provider.rs.

/// Run an async future synchronously from within a tokio runtime,
/// without risking current_thread deadlock. Spawns a fresh OS thread
/// with its own mini-runtime so the VM thread is never blocked waiting
/// for spawned tasks on the same runtime.
/// M1: KALICI paylaşımlı tokio runtime (tek şerit — tüm provider + MCP
/// köprüleri buradan geçer).
///
/// Önceki gövde her çağrıda GEÇİCİ current-thread runtime kurup `block_on`
/// sonrası düşürüyordu. Runtime düşünce üzerinde `tokio::spawn` edilmiş HER
/// task ölür: MCP client'ın `response_loop`'u (StdioRecvHalf üzerinden Child
/// süreci + stdout okuyucusunun SAHİBİ) da initialize biter bitmez
/// öldürülüyordu → sunucunun stdout'u kapanıyor, sunucu çıkıyor ve İLK
/// `tools/call` "Boru kapatılıyor (os error 232)" alıyordu. Sunucu hayatta
/// kalsa bile yanıtı okuyacak handler kalmadığından çağrı asılırdı.
///
/// Kalıcı runtime ile spawn edilen task'lar client ömrü boyunca yaşar.
/// Ayrı OS thread'inde `block_on` kalıyor: çağıran zaten bir tokio
/// runtime'ının İÇİNDEyse doğrudan `block_on` panik olur; scoped thread
/// bunu izole eder (eski davranışla aynı).
static PROVIDER_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

pub(crate) fn block_on_provider<T: Send + 'static>(
    fut: impl std::future::Future<Output = T> + Send + 'static,
) -> T {
    std::thread::scope(|s| {
        s.spawn(|| {
            let rt = PROVIDER_RUNTIME.get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .thread_name("hudhud-provider-rt")
                    .enable_all()
                    .build()
                    .expect("provider runtime build")
            });
            rt.block_on(fut)
        })
        .join()
        .unwrap()
    })
}
