use debs2021::AnalysisLocations;
use debs2021::io::{load_locations,load_batch_from};

use bencher::{Bencher,benchmark_main,benchmark_group};

fn load_testlocation_data() -> (AnalysisLocations, Vec<(f32, f32)>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let (locations, testlocations) = runtime.block_on(async {
        let locations = AnalysisLocations::new(load_locations(&root)
            .await
            .expect("Failed to load locations"));
        let batch = load_batch_from(&format!("{}/test_batch.bin", root))
            .await
            .expect("Failed to load batch");
        let testlocations = batch.current
            .iter()
            .chain(batch.lastyear.iter())
            .map(|m| (m.latitude, m.longitude))
            .take(1000)
            .collect::<Vec<_>>();
        (locations, testlocations)
    });
    (locations, testlocations)
}

fn bench_locating(bench: &mut Bencher) {
    let (locations, testlocations) = load_testlocation_data();
    bench.iter(|| {
        for (lat, lng) in &testlocations {
            let _location = locations.localize(*lat, *lng).next();
        }
    });
}

benchmark_group!(bench_locate_all, bench_locating);
benchmark_main!(bench_locate_all);
